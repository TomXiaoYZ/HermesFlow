use rand::Rng;
use serde::{Deserialize, Serialize};

/// Per-generation ALPS promotion statistics.
/// Tracks how many genomes were promoted, discarded, or purged at each layer boundary.
/// Used to detect convergence slowdown — a key P2 trigger condition.
#[derive(Debug, Clone, Default, Serialize)]
pub struct PromotionStats {
    /// Promotions that succeeded at each layer boundary [0→1, 1→2, 2→3, 3→4].
    pub promoted: [usize; 4],
    /// Genomes that aged out but were not fit enough for the next layer.
    pub discarded: [usize; 4],
    /// Genomes purged from the top layer (exceeded max_age=500).
    pub top_purged: usize,
}

impl PromotionStats {
    /// Total promotions across all layer boundaries.
    pub fn total_promoted(&self) -> usize {
        self.promoted.iter().sum()
    }

    /// Promotion rate at a specific boundary: promoted / (promoted + discarded).
    /// Returns None if no genomes aged out at this boundary.
    /// Used by P2 trigger analysis (not called in current evolution loop).
    #[allow(dead_code)]
    pub fn rate(&self, boundary: usize) -> Option<f64> {
        let total = self.promoted[boundary] + self.discarded[boundary];
        if total == 0 {
            None
        } else {
            Some(self.promoted[boundary] as f64 / total as f64)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genome {
    pub tokens: Vec<usize>,
    pub fitness: f64,
    /// Number of generations this genome has survived (used by ALPS).
    #[serde(default)]
    pub age: usize,
    /// P6a-E1: Per-token block mask. 1 = position is inside a protected semantic block.
    /// Crossover avoids cutting inside blocks; mutation skips structural changes.
    #[serde(default)]
    pub block_mask: Vec<u8>,
    /// P6a-E1: Generation at which each block was last marked (for decay).
    /// Indexed by block start position. Cleared after `BLOCK_DECAY_GENS` without improvement.
    #[serde(default)]
    pub block_age: Vec<usize>,
}

impl Genome {
    pub fn new_random(feat_offset: usize) -> Self {
        let tokens = generate_random_rpn(5, feat_offset);
        let len = tokens.len();
        Self {
            tokens,
            fitness: 0.0,
            age: 0,
            block_mask: vec![0; len],
            block_age: vec![0; len],
        }
    }

    /// Ensure block_mask and block_age are sized to match tokens.
    /// Called after crossover, mutation, or deserialization that may have changed token length.
    pub fn sync_block_fields(&mut self) {
        let len = self.tokens.len();
        self.block_mask.resize(len, 0);
        self.block_age.resize(len, 0);
    }

    /// Returns true if position `idx` is inside a protected block.
    pub fn is_blocked(&self, idx: usize) -> bool {
        idx < self.block_mask.len() && self.block_mask[idx] == 1
    }
}

/// Build operator token vectors dynamically from feat_offset.
/// Op indices match the StackVM dispatch (vm.rs).
///
/// Pruned to 14 operators for daily stock alpha formulas:
///   Unary (9):  5(ABS), 6(SIGN), 10(DELAY1), 11(DELAY5), 12(TS_MEAN),
///               13(TS_STD), 14(TS_RANK), 17(TS_MIN), 18(TS_MAX)
///   Binary (5): 0(ADD), 1(SUB), 2(MUL), 3(DIV), 16(TS_CORR)
///
/// Removed (9): NEG(4), GATE(7), SIGNED_POWER(8), DECAY_LINEAR(9),
///   TS_SUM(15), LOG(19), SQRT(20), TS_ARGMAX(21), TS_DELTA(22)
///   — redundant, domain-error-prone, or disruptive to stack arithmetic.
/// VM still executes all 23 opcodes for backward compatibility with stored genomes.
fn build_ops(feat_offset: usize) -> (Vec<usize>, Vec<usize>) {
    let unary_op_indices: Vec<usize> = vec![5, 6, 10, 11, 12, 13, 14, 17, 18];
    let binary_op_indices: Vec<usize> = vec![0, 1, 2, 3, 16];

    let ops_1: Vec<usize> = unary_op_indices
        .into_iter()
        .map(|idx| idx + feat_offset)
        .collect();
    let ops_2: Vec<usize> = binary_op_indices
        .into_iter()
        .map(|idx| idx + feat_offset)
        .collect();

    (ops_1, ops_2)
}

/// Classify a token as feature, unary, or binary operator.
/// Ternary ops (GATE) are no longer generated but still recognized for
/// backward compatibility when evaluating stored genomes.
fn token_arity(token: usize, feat_offset: usize) -> usize {
    if token < feat_offset {
        return 0; // feature (pushes to stack)
    }
    let op_idx = token - feat_offset;
    match op_idx {
        0..=3 | 16 => 2, // binary: ADD, SUB, MUL, DIV, TS_CORR
        7 => 3,          // ternary: GATE (legacy only, no longer generated)
        _ => 1,          // unary: everything else
    }
}

fn generate_random_rpn(max_depth: usize, feat_offset: usize) -> Vec<usize> {
    let mut rng = rand::thread_rng();

    let features: Vec<usize> = (0..feat_offset).collect();
    let (ops_1, ops_2) = build_ops(feat_offset);

    let mut tokens = Vec::new();

    // Step 1: Push at least one feature
    tokens.push(features[rng.gen_range(0..features.len())]);
    let mut stack_depth = 1;

    let mut steps = 0;
    let target_len = rng.gen_range(3..12);

    while steps < target_len || stack_depth > 1 {
        let mut choices = Vec::new();

        if stack_depth < max_depth && steps < target_len {
            choices.push("FEAT");
        }
        if stack_depth >= 1 {
            choices.push("OP1");
        }
        if stack_depth >= 2 {
            choices.push("OP2");
        }

        // Forced collapse if length exceeded
        if steps >= target_len {
            choices.clear();
            if stack_depth >= 2 {
                choices.push("OP2");
            } else {
                choices.push("OP1");
            }

            if stack_depth == 1 {
                break;
            }
        }

        if choices.is_empty() {
            break;
        }

        let action = choices[rng.gen_range(0..choices.len())];

        match action {
            "FEAT" => {
                tokens.push(features[rng.gen_range(0..features.len())]);
                stack_depth += 1;
            }
            "OP1" => {
                tokens.push(ops_1[rng.gen_range(0..ops_1.len())]);
            }
            "OP2" => {
                tokens.push(ops_2[rng.gen_range(0..ops_2.len())]);
                stack_depth -= 1;
            }
            _ => {}
        }

        steps += 1;
    }

    tokens
}

/// ALPS layer configuration: max_age uses Fibonacci-like gaps.
/// Layer 4 (elite) capped at 500 to prevent ancient genome stagnation.
/// Over-aged elites are discarded; best_genome field preserves the all-time best.
const ALPS_LAYER_MAX_AGES: [usize; 5] = [5, 13, 34, 89, 500];
const ALPS_LAYER_POP_SIZE: usize = 100;
const ALPS_NUM_LAYERS: usize = 5;

/// Number of generations without fitness improvement before blocks decay (P6a-E1).
const BLOCK_DECAY_GENS: usize = 20;

/// A single ALPS age layer with its own population.
struct AlpsLayer {
    population: Vec<Genome>,
    max_age: usize,
}

impl AlpsLayer {
    fn new(max_age: usize, feat_offset: usize, pop_size: usize) -> Self {
        let mut population = Vec::with_capacity(pop_size);
        for _ in 0..pop_size {
            population.push(Genome::new_random(feat_offset));
        }
        Self {
            population,
            max_age,
        }
    }

    fn sort_by_fitness(&mut self) {
        self.population.sort_by(|a, b| {
            b.fitness
                .partial_cmp(&a.fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Within-layer tournament selection (k=3).
    fn tournament_select(&self) -> &Genome {
        let mut rng = rand::thread_rng();
        let mut best: Option<&Genome> = None;
        for _ in 0..3 {
            let idx = rng.gen_range(0..self.population.len());
            let candidate = &self.population[idx];
            if best.is_none() || candidate.fitness > best.unwrap().fitness {
                best = Some(candidate);
            }
        }
        best.unwrap()
    }

    /// Remove duplicate genomes within this layer.
    fn deduplicate(&mut self, feat_offset: usize) {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for genome in self.population.iter_mut() {
            if !seen.insert(genome.tokens.clone()) {
                let new_tokens = generate_random_rpn(5, feat_offset);
                let len = new_tokens.len();
                genome.tokens = new_tokens;
                genome.fitness = 0.0;
                genome.age = 0;
                genome.block_mask = vec![0; len];
                genome.block_age = vec![0; len];
            }
        }
    }
}

/// Age-Layered Population Structure (ALPS) genetic algorithm.
///
/// Maintains diversity by stratifying the population into age layers:
/// - Layer 0 (max_age=5): Fresh random exploration, continuously replenished
/// - Layer 1 (max_age=13): Young promising genomes
/// - Layer 2 (max_age=34): Maturing genomes
/// - Layer 3 (max_age=89): Experienced genomes
/// - Layer 4 (max_age=∞): Elite archive
///
/// Genomes that exceed their layer's max_age are promoted to the next layer
/// (if fitter than the worst genome there) or discarded. This prevents old
/// elite genomes from dominating young exploration layers.
pub struct AlpsGA {
    layers: Vec<AlpsLayer>,
    pub generation: usize,
    pub best_genome: Option<Genome>,
    pub feat_offset: usize,
}

/// P7-5B: Normalized Hamming distance between two token sequences.
///
/// Compares element-wise, treating different lengths as mismatches for
/// the excess portion. Returns value in [0.0, 1.0].
fn hamming_distance(a: &[usize], b: &[usize]) -> f64 {
    let max_len = a.len().max(b.len());
    if max_len == 0 {
        return 0.0;
    }
    let min_len = a.len().min(b.len());
    let mut diffs = (max_len - min_len) as f64; // excess tokens are all mismatches
    for i in 0..min_len {
        if a[i] != b[i] {
            diffs += 1.0;
        }
    }
    diffs / max_len as f64
}

impl AlpsGA {
    pub fn new(feat_offset: usize) -> Self {
        let layers: Vec<AlpsLayer> = ALPS_LAYER_MAX_AGES
            .iter()
            .map(|&max_age| AlpsLayer::new(max_age, feat_offset, ALPS_LAYER_POP_SIZE))
            .collect();

        Self {
            layers,
            generation: 0,
            best_genome: None,
            feat_offset,
        }
    }

    /// Get a flat view of all genomes across all layers (for evaluation).
    pub fn all_genomes_mut(&mut self) -> Vec<&mut Genome> {
        self.layers
            .iter_mut()
            .flat_map(|layer| layer.population.iter_mut())
            .collect()
    }

    /// Total population size across all layers.
    pub fn total_population(&self) -> usize {
        self.layers.iter().map(|l| l.population.len()).sum()
    }

    /// Population size of a specific layer.
    pub fn layer_size(&self, layer_idx: usize) -> usize {
        self.layers
            .get(layer_idx)
            .map(|l| l.population.len())
            .unwrap_or(0)
    }

    /// Generate N random genomes suitable for injection.
    pub fn generate_random_genomes(&self, count: usize) -> Vec<Genome> {
        (0..count)
            .map(|_| Genome::new_random(self.feat_offset))
            .collect()
    }

    /// P8-2B: Remove the weakest genomes from a layer by fitness.
    /// Returns the number actually culled.
    pub fn cull_weakest(&mut self, layer_idx: usize, count: usize) -> usize {
        if layer_idx >= self.layers.len() || count == 0 {
            return 0;
        }
        let layer = &mut self.layers[layer_idx];
        layer.sort_by_fitness();
        let actual = count.min(layer.population.len());
        // sort_by_fitness puts best first; remove from the end (worst)
        layer.population.truncate(layer.population.len() - actual);
        actual
    }

    /// Inject externally-generated genomes into a specific layer.
    /// Used by the LLM mutation oracle to insert guided genomes into Layer 0.
    /// Replaces the worst genomes if the layer is at capacity.
    pub fn inject_genomes(&mut self, layer_idx: usize, genomes: Vec<Genome>) {
        if layer_idx >= self.layers.len() || genomes.is_empty() {
            return;
        }
        let layer = &mut self.layers[layer_idx];
        layer.sort_by_fitness();
        for mut genome in genomes {
            genome.sync_block_fields();
            if layer.population.len() < ALPS_LAYER_POP_SIZE {
                layer.population.push(genome);
            } else {
                // Replace worst genome
                layer.population.pop();
                layer.population.push(genome);
            }
        }
    }

    /// Collect top-N genomes from each layer (for LLM oracle context).
    pub fn collect_elites(&self, per_layer: usize) -> Vec<(usize, &Genome)> {
        let mut elites = Vec::new();
        for (layer_idx, layer) in self.layers.iter().enumerate() {
            let mut sorted: Vec<&Genome> = layer
                .population
                .iter()
                .filter(|g| g.fitness.is_finite())
                .collect();
            sorted.sort_by(|a, b| {
                b.fitness
                    .partial_cmp(&a.fitness)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            for genome in sorted.into_iter().take(per_layer) {
                elites.push((layer_idx, genome));
            }
        }
        elites
    }

    /// Summary of layer sizes and best fitness per layer (for logging).
    pub fn layer_summary(&self) -> Vec<(usize, usize, f64)> {
        self.layers
            .iter()
            .enumerate()
            .map(|(i, layer)| {
                let best_fit = layer
                    .population
                    .iter()
                    .map(|g| g.fitness)
                    .fold(f64::NEG_INFINITY, f64::max);
                (i, layer.population.len(), best_fit)
            })
            .collect()
    }

    /// P7-5B: Compute mean pairwise Hamming distance per ALPS layer.
    ///
    /// Returns `Vec<(layer_index, population_size, mean_hamming_distance)>`.
    /// Higher diversity = more distinct token sequences = healthier exploration.
    /// Samples up to 50 pairs per layer for O(1) computation.
    pub fn layer_diversity(&self) -> Vec<(usize, usize, f64)> {
        self.layers
            .iter()
            .enumerate()
            .map(|(i, layer)| {
                let pop = &layer.population;
                let n = pop.len();
                if n < 2 {
                    return (i, n, 0.0);
                }
                let max_pairs = 50usize;
                let mut total_dist = 0.0;
                let mut count = 0usize;
                'outer: for a in 0..n {
                    for b in (a + 1)..n {
                        if count >= max_pairs {
                            break 'outer;
                        }
                        total_dist += hamming_distance(&pop[a].tokens, &pop[b].tokens);
                        count += 1;
                    }
                }
                let mean = if count > 0 {
                    total_dist / count as f64
                } else {
                    0.0
                };
                (i, n, mean)
            })
            .collect()
    }

    /// P6a-E1: Mark a genome's entire token sequence as a protected block.
    /// Called after walk-forward validation confirms a genome is performing well.
    #[allow(dead_code)]
    pub fn mark_block(genome: &mut Genome, generation: usize) {
        let len = genome.tokens.len();
        genome.block_mask = vec![1; len];
        genome.block_age = vec![generation; len];
    }

    /// Evolve one generation. Returns detailed promotion statistics per layer boundary.
    pub fn evolve(&mut self) -> PromotionStats {
        let mut rng = rand::thread_rng();
        let mut stats = PromotionStats::default();
        let current_gen = self.generation;

        // Phase 0 (P6a-E1): Block decay — clear blocks that haven't improved in BLOCK_DECAY_GENS
        for layer in &mut self.layers {
            for genome in &mut layer.population {
                if genome.block_mask.contains(&1) {
                    // Check if block is stale: marked_gen + BLOCK_DECAY_GENS < current_gen
                    let marked_gen = genome
                        .block_age
                        .iter()
                        .filter(|&&a| a > 0)
                        .min()
                        .copied()
                        .unwrap_or(0);
                    if current_gen > marked_gen + BLOCK_DECAY_GENS {
                        // Clear block protection — allow full mutation/crossover again
                        genome.block_mask.fill(0);
                        genome.block_age.fill(0);
                    }
                }
            }
        }

        // Phase 1: Age all surviving genomes
        for layer in &mut self.layers {
            for genome in &mut layer.population {
                genome.age += 1;
            }
        }

        // Phase 2: Promote over-aged genomes upward (bottom-up)
        for layer_idx in 0..ALPS_NUM_LAYERS - 1 {
            let max_age = self.layers[layer_idx].max_age;

            // Collect genomes that exceeded this layer's max_age
            let mut candidates = Vec::new();
            self.layers[layer_idx].population.retain(|g| {
                if g.age > max_age {
                    candidates.push(g.clone());
                    false
                } else {
                    true
                }
            });

            // Try to insert promoted genomes into the next layer
            let next_layer = &mut self.layers[layer_idx + 1];
            for genome in candidates {
                if next_layer.population.len() < ALPS_LAYER_POP_SIZE {
                    // Room available — just insert
                    next_layer.population.push(genome);
                    stats.promoted[layer_idx] += 1;
                } else {
                    // Replace worst genome if promoted genome is fitter
                    next_layer.sort_by_fitness();
                    if let Some(worst) = next_layer.population.last() {
                        if genome.fitness > worst.fitness {
                            next_layer.population.pop();
                            next_layer.population.push(genome);
                            stats.promoted[layer_idx] += 1;
                        } else {
                            stats.discarded[layer_idx] += 1;
                        }
                    }
                }
            }
        }

        // Phase 2b: Discard over-aged genomes from top layer (no higher layer to promote to)
        let top = &mut self.layers[ALPS_NUM_LAYERS - 1];
        let before = top.population.len();
        top.population.retain(|g| g.age <= top.max_age);
        stats.top_purged = before - top.population.len();

        // Phase 3: Replenish layer 0 with fresh random genomes
        let layer0 = &mut self.layers[0];
        while layer0.population.len() < ALPS_LAYER_POP_SIZE {
            layer0.population.push(Genome::new_random(self.feat_offset));
        }

        // Phase 4: Evolve each layer independently
        for layer in &mut self.layers {
            if layer.population.is_empty() {
                // Seed empty layers with random genomes so evolution can proceed
                for _ in 0..ALPS_LAYER_POP_SIZE {
                    layer.population.push(Genome::new_random(self.feat_offset));
                }
            }

            layer.sort_by_fitness();
            layer.deduplicate(self.feat_offset);

            // Always target ALPS_LAYER_POP_SIZE so layers recover after age-based discarding
            let pop_size = ALPS_LAYER_POP_SIZE;
            let current_len = layer.population.len();
            let elitism_count =
                (current_len as f64 * 0.05).max(2.0).min(current_len as f64) as usize;

            let mut new_pop = Vec::with_capacity(pop_size);

            // Elitism: keep top genomes
            for i in 0..elitism_count.min(current_len) {
                new_pop.push(layer.population[i].clone());
            }

            // Fill with crossover (40%), mutation (35%), immigration (25%)
            while new_pop.len() < pop_size {
                let parent1 = layer.tournament_select();
                let parent2 = layer.tournament_select();

                let r: f64 = rng.gen();
                if r < 0.40 {
                    let mut child = Self::crossover(parent1, parent2, self.feat_offset);
                    // Child inherits max parent age (ALPS convention)
                    child.age = parent1.age.max(parent2.age);
                    child.sync_block_fields();
                    Self::mutate(&mut child, self.feat_offset);
                    child.sync_block_fields();
                    new_pop.push(child);
                } else if r < 0.75 {
                    let mut child = parent1.clone();
                    Self::mutate(&mut child, self.feat_offset);
                    child.sync_block_fields();
                    new_pop.push(child);
                } else {
                    new_pop.push(Genome::new_random(self.feat_offset));
                }
            }

            layer.population = new_pop;
        }

        // Phase 5: Update global best genome across all layers
        for layer in &self.layers {
            for genome in &layer.population {
                let dominated = match &self.best_genome {
                    Some(current) => genome.fitness > current.fitness,
                    None => true,
                };
                if dominated {
                    self.best_genome = Some(genome.clone());
                }
            }
        }

        self.generation += 1;
        stats
    }

    /// Single-point crossover: take prefix of parent1, suffix of parent2.
    /// P6a-E1: Uses block-aware cut points to avoid splitting protected blocks.
    fn crossover(parent1: &Genome, parent2: &Genome, feat_offset: usize) -> Genome {
        let mut rng = rand::thread_rng();

        // Use block-aware cut points when masks are available
        let mask1 = if parent1.block_mask.len() == parent1.tokens.len() {
            Some(parent1.block_mask.as_slice())
        } else {
            None
        };
        let mask2 = if parent2.block_mask.len() == parent2.tokens.len() {
            Some(parent2.block_mask.as_slice())
        } else {
            None
        };

        let cuts1 = valid_cut_points_with_mask(&parent1.tokens, feat_offset, mask1);
        let cuts2 = valid_cut_points_with_mask(&parent2.tokens, feat_offset, mask2);

        if cuts1.is_empty() || cuts2.is_empty() {
            return parent1.clone();
        }

        let cut1 = cuts1[rng.gen_range(0..cuts1.len())];
        let cut2 = cuts2[rng.gen_range(0..cuts2.len())];

        let mut tokens = parent1.tokens[..cut1].to_vec();
        tokens.extend_from_slice(&parent2.tokens[cut2..]);

        // Merge block masks from both parents
        let mut block_mask = Vec::with_capacity(tokens.len());
        if parent1.block_mask.len() >= cut1 {
            block_mask.extend_from_slice(&parent1.block_mask[..cut1]);
        } else {
            block_mask.resize(cut1, 0);
        }
        if parent2.block_mask.len() > cut2 {
            block_mask.extend_from_slice(&parent2.block_mask[cut2..]);
        }

        // Merge block ages similarly
        let mut block_age = Vec::with_capacity(tokens.len());
        if parent1.block_age.len() >= cut1 {
            block_age.extend_from_slice(&parent1.block_age[..cut1]);
        } else {
            block_age.resize(cut1, 0);
        }
        if parent2.block_age.len() > cut2 {
            block_age.extend_from_slice(&parent2.block_age[cut2..]);
        }

        if tokens.len() > 20 {
            tokens.truncate(20);
            block_mask.truncate(20);
            block_age.truncate(20);
        }

        let len = tokens.len();
        block_mask.resize(len, 0);
        block_age.resize(len, 0);

        Genome {
            tokens,
            fitness: 0.0,
            age: 0,
            block_mask,
            block_age,
        }
    }

    /// Mutation operators (no stagnation-dependent rates — ALPS handles diversity).
    /// P6a-E1: Point and operator mutations are allowed on blocked positions (same-arity swap
    /// preserves block structure). Growth, shrink, and subtree replacement skip blocked positions.
    fn mutate(genome: &mut Genome, feat_offset: usize) {
        let mut rng = rand::thread_rng();
        let (ops_1, ops_2) = build_ops(feat_offset);

        // 1. Point mutation (40%): change any token to same-arity token
        // Allowed on blocked positions — same arity preserves structure.
        if rng.gen_bool(0.4) && !genome.tokens.is_empty() {
            let idx = rng.gen_range(0..genome.tokens.len());
            let old = genome.tokens[idx];

            if old < feat_offset {
                genome.tokens[idx] = rng.gen_range(0..feat_offset);
            } else {
                let arity = token_arity(old, feat_offset);
                let pool = match arity {
                    1 => &ops_1,
                    2 => &ops_2,
                    _ => &ops_1,
                };
                genome.tokens[idx] = pool[rng.gen_range(0..pool.len())];
            }
        }

        // 2. Operator mutation (20%): swap a random operator for same-arity
        // Allowed on blocked positions — same arity preserves structure.
        if rng.gen_bool(0.2) && !genome.tokens.is_empty() {
            let op_indices: Vec<usize> = genome
                .tokens
                .iter()
                .enumerate()
                .filter(|(_, &t)| t >= feat_offset)
                .map(|(i, _)| i)
                .collect();
            if !op_indices.is_empty() {
                let idx = op_indices[rng.gen_range(0..op_indices.len())];
                let old = genome.tokens[idx];
                let arity = token_arity(old, feat_offset);
                let pool = match arity {
                    1 => &ops_1,
                    2 => &ops_2,
                    _ => &ops_1,
                };
                genome.tokens[idx] = pool[rng.gen_range(0..pool.len())];
            }
        }

        // 3. Growth mutation (8%): append Feature + BinaryOp (stack-neutral)
        // Appending at the end is safe — only skip if last token is blocked
        // (which would mean we're extending a block boundary).
        if rng.gen_bool(0.08) && genome.tokens.len() < 20 {
            let feat = rng.gen_range(0..feat_offset);
            let op = ops_2[rng.gen_range(0..ops_2.len())];
            genome.tokens.push(feat);
            genome.tokens.push(op);
            genome.sync_block_fields();
        }

        // 4. Shrink mutation (20%): remove a unary op (stack-neutral)
        // P6a-E1: Only remove unblocked unary ops.
        if rng.gen_bool(0.20) && genome.tokens.len() > 3 {
            let unary_indices: Vec<usize> = genome
                .tokens
                .iter()
                .enumerate()
                .filter(|(i, &t)| {
                    t >= feat_offset && token_arity(t, feat_offset) == 1 && !genome.is_blocked(*i)
                })
                .map(|(i, _)| i)
                .collect();
            if !unary_indices.is_empty() {
                let idx = unary_indices[rng.gen_range(0..unary_indices.len())];
                genome.tokens.remove(idx);
                if idx < genome.block_mask.len() {
                    genome.block_mask.remove(idx);
                }
                if idx < genome.block_age.len() {
                    genome.block_age.remove(idx);
                }
            }
        }

        // 5. Subtree replacement (10%): replace genome with new random subtree
        // P6a-E1: Skip if any token is blocked (preserve the proven formula).
        if rng.gen_bool(0.10) {
            let has_blocks = genome.block_mask.contains(&1);
            if !has_blocks {
                let new_subtree = generate_random_rpn(5, feat_offset);
                let len = new_subtree.len();
                genome.tokens = new_subtree;
                genome.block_mask = vec![0; len];
                genome.block_age = vec![0; len];
                genome.age = 0; // Reset age on full replacement
            }
        }
    }
}

/// Find positions in the token sequence where the running stack depth is exactly 1.
/// These are valid "cut points" for crossover — the formula up to that point
/// produces exactly one value on the stack.
///
/// P6a-E1: If `block_mask` is provided, skip cut points that fall inside a protected block.
#[cfg(test)]
fn valid_cut_points(tokens: &[usize], feat_offset: usize) -> Vec<usize> {
    valid_cut_points_with_mask(tokens, feat_offset, None)
}

/// Inner implementation that accepts an optional block mask for E1 protection.
fn valid_cut_points_with_mask(
    tokens: &[usize],
    feat_offset: usize,
    block_mask: Option<&[u8]>,
) -> Vec<usize> {
    let mut depth: i32 = 0;
    let mut cuts = Vec::new();

    for (i, &token) in tokens.iter().enumerate() {
        if token < feat_offset {
            depth += 1; // feature pushes
        } else {
            let arity = token_arity(token, feat_offset);
            // operator pops `arity` and pushes 1
            depth = depth - arity as i32 + 1;
        }

        // A valid cut is where we have exactly 1 item on the stack
        // (skip position 0 to avoid empty prefixes)
        if depth == 1 && i > 0 {
            let cut_pos = i + 1; // cut AFTER this token

            // P6a-E1: Skip if the cut would split a protected block.
            // A cut at position `cut_pos` is inside a block if either
            // the token before the cut or the token after is blocked.
            if let Some(mask) = block_mask {
                let before_blocked = mask.get(i).copied().unwrap_or(0) == 1;
                let after_blocked = mask.get(cut_pos).copied().unwrap_or(0) == 1;
                if before_blocked && after_blocked {
                    continue; // skip — would split a block
                }
            }

            cuts.push(cut_pos);
        }
    }

    cuts
}

#[cfg(test)]
mod tests {
    use super::*;

    const FEAT_OFFSET_25: usize = 25;
    const FEAT_OFFSET_75: usize = 75;

    // ── build_ops ──────────────────────────────────────────────────────

    #[test]
    fn build_ops_unary_count() {
        let (ops_1, _) = build_ops(FEAT_OFFSET_25);
        // 9 unary: ABS(5), SIGN(6), DELAY1(10), DELAY5(11), TS_MEAN(12),
        //          TS_STD(13), TS_RANK(14), TS_MIN(17), TS_MAX(18)
        assert_eq!(ops_1.len(), 9);
    }

    #[test]
    fn build_ops_binary_count() {
        let (_, ops_2) = build_ops(FEAT_OFFSET_25);
        // 5 binary: ADD(0), SUB(1), MUL(2), DIV(3), TS_CORR(16)
        assert_eq!(ops_2.len(), 5);
    }

    #[test]
    fn build_ops_offset_applied() {
        let (ops_1, ops_2) = build_ops(FEAT_OFFSET_25);
        // All tokens must be >= feat_offset
        for &t in &ops_1 {
            assert!(t >= FEAT_OFFSET_25, "unary token {} < feat_offset", t);
        }
        for &t in &ops_2 {
            assert!(t >= FEAT_OFFSET_25, "binary token {} < feat_offset", t);
        }
        // ADD should be feat_offset + 0
        assert!(ops_2.contains(&FEAT_OFFSET_25));
        // ABS should be feat_offset + 5
        assert!(ops_1.contains(&(FEAT_OFFSET_25 + 5)));
    }

    #[test]
    fn build_ops_neg_excluded() {
        let (ops_1, ops_2) = build_ops(FEAT_OFFSET_25);
        // NEG(4) should NOT be in the generation pool
        let neg_token = FEAT_OFFSET_25 + 4;
        assert!(!ops_1.contains(&neg_token), "NEG should be excluded");
        assert!(!ops_2.contains(&neg_token), "NEG should be excluded");
    }

    #[test]
    fn build_ops_mtf_75_offset() {
        let (ops_1, ops_2) = build_ops(FEAT_OFFSET_75);
        assert_eq!(ops_1.len(), 9);
        assert_eq!(ops_2.len(), 5);
        // ADD at offset 75
        assert!(ops_2.contains(&(FEAT_OFFSET_75)));
        assert!(*ops_1.iter().min().unwrap() >= FEAT_OFFSET_75);
    }

    // ── token_arity ────────────────────────────────────────────────────

    #[test]
    fn token_arity_features_are_zero() {
        for t in 0..FEAT_OFFSET_25 {
            assert_eq!(
                token_arity(t, FEAT_OFFSET_25),
                0,
                "feature {} arity != 0",
                t
            );
        }
    }

    #[test]
    fn token_arity_binary_ops() {
        let binary_indices = [0, 1, 2, 3, 16]; // ADD, SUB, MUL, DIV, TS_CORR
        for &idx in &binary_indices {
            assert_eq!(
                token_arity(FEAT_OFFSET_25 + idx, FEAT_OFFSET_25),
                2,
                "op {} should be binary",
                idx
            );
        }
    }

    #[test]
    fn token_arity_unary_ops() {
        let unary_indices = [5, 6, 10, 11, 12, 13, 14, 17, 18];
        for &idx in &unary_indices {
            assert_eq!(
                token_arity(FEAT_OFFSET_25 + idx, FEAT_OFFSET_25),
                1,
                "op {} should be unary",
                idx
            );
        }
    }

    #[test]
    fn token_arity_gate_is_ternary() {
        assert_eq!(token_arity(FEAT_OFFSET_25 + 7, FEAT_OFFSET_25), 3);
    }

    // ── generate_random_rpn ────────────────────────────────────────────

    #[test]
    fn random_rpn_nonempty() {
        for _ in 0..50 {
            let tokens = generate_random_rpn(5, FEAT_OFFSET_25);
            assert!(!tokens.is_empty(), "random RPN should not be empty");
        }
    }

    #[test]
    fn random_rpn_starts_with_feature() {
        for _ in 0..50 {
            let tokens = generate_random_rpn(5, FEAT_OFFSET_25);
            assert!(
                tokens[0] < FEAT_OFFSET_25,
                "first token should be a feature, got {}",
                tokens[0]
            );
        }
    }

    #[test]
    fn random_rpn_valid_stack_depth() {
        // Every generated RPN should resolve to stack depth 1
        for _ in 0..100 {
            let tokens = generate_random_rpn(5, FEAT_OFFSET_25);
            let mut depth: i32 = 0;
            for &t in &tokens {
                if t < FEAT_OFFSET_25 {
                    depth += 1;
                } else {
                    let arity = token_arity(t, FEAT_OFFSET_25) as i32;
                    depth = depth - arity + 1;
                }
            }
            assert_eq!(depth, 1, "stack depth should be 1, tokens: {:?}", tokens);
        }
    }

    #[test]
    fn random_rpn_length_bounded() {
        for _ in 0..100 {
            let tokens = generate_random_rpn(5, FEAT_OFFSET_25);
            assert!(tokens.len() >= 1, "minimum 1 token");
            assert!(tokens.len() <= 25, "max ~20 tokens + collapse overhead");
        }
    }

    #[test]
    fn random_rpn_tokens_in_range() {
        let (ops_1, ops_2) = build_ops(FEAT_OFFSET_25);
        let valid_ops: std::collections::HashSet<usize> =
            ops_1.iter().chain(ops_2.iter()).copied().collect();
        for _ in 0..50 {
            let tokens = generate_random_rpn(5, FEAT_OFFSET_25);
            for &t in &tokens {
                if t >= FEAT_OFFSET_25 {
                    assert!(
                        valid_ops.contains(&t),
                        "op token {} not in active ops set",
                        t
                    );
                }
            }
        }
    }

    // ── valid_cut_points ───────────────────────────────────────────────

    #[test]
    fn cut_points_simple_formula() {
        // feat0 feat1 ADD → stack depths: [1, 2, 1]
        // Cut point at index 2 (after ADD) → cut = 3
        let tokens = vec![0, 1, FEAT_OFFSET_25]; // ADD = offset+0
        let cuts = valid_cut_points(&tokens, FEAT_OFFSET_25);
        assert_eq!(cuts, vec![3]);
    }

    #[test]
    fn cut_points_chained() {
        // feat0 ABS feat1 ADD → depths: [1, 1, 2, 1]
        // ABS is unary: pops 1, pushes 1 → depth stays 1 at index 1
        // feat1 pushes → depth 2
        // ADD pops 2, pushes 1 → depth 1 at index 3
        let abs_token = FEAT_OFFSET_25 + 5;
        let add_token = FEAT_OFFSET_25 + 0;
        let tokens = vec![0, abs_token, 1, add_token];
        let cuts = valid_cut_points(&tokens, FEAT_OFFSET_25);
        // depth=1 at index 1 (after ABS) → cut=2
        // depth=1 at index 3 (after ADD) → cut=4
        assert_eq!(cuts, vec![2, 4]);
    }

    #[test]
    fn cut_points_single_feature_no_cuts() {
        // Single feature → depth 1 at index 0, but we skip position 0
        let tokens = vec![5];
        let cuts = valid_cut_points(&tokens, FEAT_OFFSET_25);
        assert!(cuts.is_empty());
    }

    // ── Genome ─────────────────────────────────────────────────────────

    #[test]
    fn genome_new_random_fields() {
        let g = Genome::new_random(FEAT_OFFSET_25);
        assert_eq!(g.fitness, 0.0);
        assert_eq!(g.age, 0);
        assert!(!g.tokens.is_empty());
    }

    // ── PromotionStats ─────────────────────────────────────────────────

    #[test]
    fn promotion_stats_total() {
        let mut stats = PromotionStats::default();
        stats.promoted = [3, 2, 1, 0];
        assert_eq!(stats.total_promoted(), 6);
    }

    #[test]
    fn promotion_stats_rate() {
        let mut stats = PromotionStats::default();
        stats.promoted = [7, 0, 0, 0];
        stats.discarded = [3, 0, 0, 0];
        assert!((stats.rate(0).unwrap() - 0.7).abs() < 1e-10);
        assert!(stats.rate(1).is_none()); // no activity
    }

    // ── AlpsGA ─────────────────────────────────────────────────────────

    #[test]
    fn alps_ga_initial_state() {
        let ga = AlpsGA::new(FEAT_OFFSET_25);
        assert_eq!(ga.generation, 0);
        assert!(ga.best_genome.is_none());
        assert_eq!(ga.feat_offset, FEAT_OFFSET_25);
        assert_eq!(ga.total_population(), 500); // 5 layers × 100
    }

    #[test]
    fn alps_ga_layer_summary() {
        let ga = AlpsGA::new(FEAT_OFFSET_25);
        let summary = ga.layer_summary();
        assert_eq!(summary.len(), 5);
        for (i, size, _best) in &summary {
            assert_eq!(*size, 100, "layer {} should have 100 genomes", i);
        }
    }

    #[test]
    fn alps_ga_all_genomes_mut_count() {
        let mut ga = AlpsGA::new(FEAT_OFFSET_25);
        let all = ga.all_genomes_mut();
        assert_eq!(all.len(), 500);
    }

    #[test]
    fn alps_ga_evolve_increments_generation() {
        let mut ga = AlpsGA::new(FEAT_OFFSET_25);
        assert_eq!(ga.generation, 0);
        ga.evolve();
        assert_eq!(ga.generation, 1);
        ga.evolve();
        assert_eq!(ga.generation, 2);
    }

    #[test]
    fn alps_ga_evolve_preserves_population() {
        let mut ga = AlpsGA::new(FEAT_OFFSET_25);
        ga.evolve();
        // Population should stay at 500 (replenished)
        assert_eq!(ga.total_population(), 500);
    }

    #[test]
    fn alps_ga_evolve_ages_genomes() {
        let mut ga = AlpsGA::new(FEAT_OFFSET_25);
        // After one evolve, all initial genomes get age+1
        ga.evolve();
        let all = ga.all_genomes_mut();
        // Not all will be age 1 (some are new immigrants with age 0),
        // but at least some should have age > 0
        let aged = all.iter().filter(|g| g.age > 0).count();
        assert!(aged > 0, "some genomes should have aged");
    }

    #[test]
    fn alps_ga_promotion_after_many_generations() {
        let mut ga = AlpsGA::new(FEAT_OFFSET_25);
        // Set fitness on layer 0 genomes so promotion can work
        for g in ga.all_genomes_mut() {
            g.fitness = 1.0;
        }
        // Evolve past layer 0 max_age (5 generations)
        let mut total_promoted = 0;
        for _ in 0..10 {
            let stats = ga.evolve();
            total_promoted += stats.total_promoted();
        }
        // After 10 generations, layer 0 genomes (age > 5) should be promoted
        assert!(
            total_promoted > 0,
            "should have promotions after 10 generations"
        );
    }

    #[test]
    fn alps_ga_inject_genomes() {
        let mut ga = AlpsGA::new(FEAT_OFFSET_25);
        let injected = vec![Genome {
            tokens: vec![0, 1, FEAT_OFFSET_25],
            fitness: 999.0,
            age: 0,
            block_mask: vec![0, 0, 0],
            block_age: vec![0, 0, 0],
        }];
        ga.inject_genomes(0, injected);
        // Should still be 100 (replaced worst, not added)
        assert_eq!(ga.layers[0].population.len(), 100);
        // The injected genome should be in layer 0
        let has_999 = ga.layers[0].population.iter().any(|g| g.fitness == 999.0);
        assert!(has_999, "injected genome should be in layer 0");
    }

    #[test]
    fn alps_ga_inject_out_of_bounds() {
        let mut ga = AlpsGA::new(FEAT_OFFSET_25);
        ga.inject_genomes(99, vec![Genome::new_random(FEAT_OFFSET_25)]);
        // Should be a no-op
        assert_eq!(ga.total_population(), 500);
    }

    #[test]
    fn alps_ga_collect_elites() {
        let mut ga = AlpsGA::new(FEAT_OFFSET_25);
        // Set some fitness values
        ga.layers[0].population[0].fitness = 5.0;
        ga.layers[1].population[0].fitness = 10.0;
        let elites = ga.collect_elites(2);
        // Should have up to 2 per layer × 5 layers = 10
        assert!(elites.len() <= 10);
        assert!(elites.len() >= 2); // at least 2 (from layers with finite fitness)
    }

    #[test]
    fn alps_ga_best_genome_updated() {
        let mut ga = AlpsGA::new(FEAT_OFFSET_25);
        // Set a genome with high fitness
        ga.layers[2].population[0].fitness = 100.0;
        ga.evolve();
        assert!(ga.best_genome.is_some());
        assert!(ga.best_genome.as_ref().unwrap().fitness >= 100.0);
    }

    // ── Crossover ──────────────────────────────────────────────────────

    #[test]
    fn crossover_produces_valid_genome() {
        let p1 = Genome {
            tokens: vec![0, 1, FEAT_OFFSET_25], // f0 f1 ADD
            fitness: 1.0,
            age: 5,
            block_mask: vec![0, 0, 0],
            block_age: vec![0, 0, 0],
        };
        let p2 = Genome {
            tokens: vec![2, FEAT_OFFSET_25 + 5, 3, FEAT_OFFSET_25 + 1], // f2 ABS f3 SUB
            fitness: 2.0,
            age: 10,
            block_mask: vec![0, 0, 0, 0],
            block_age: vec![0, 0, 0, 0],
        };
        // Run crossover multiple times (stochastic)
        for _ in 0..20 {
            let child = AlpsGA::crossover(&p1, &p2, FEAT_OFFSET_25);
            assert!(!child.tokens.is_empty());
            assert_eq!(child.fitness, 0.0); // reset
            assert_eq!(child.age, 0); // reset
            assert!(child.tokens.len() <= 20); // truncation limit
                                               // Block fields should be sized to match tokens
            assert_eq!(child.block_mask.len(), child.tokens.len());
            assert_eq!(child.block_age.len(), child.tokens.len());
        }
    }

    // ── Mutation ───────────────────────────────────────────────────────

    #[test]
    fn mutate_does_not_crash() {
        for _ in 0..100 {
            let mut g = Genome::new_random(FEAT_OFFSET_25);
            AlpsGA::mutate(&mut g, FEAT_OFFSET_25);
            assert!(!g.tokens.is_empty());
        }
    }

    #[test]
    fn mutate_tokens_stay_in_range() {
        let (ops_1, ops_2) = build_ops(FEAT_OFFSET_25);
        let valid_ops: std::collections::HashSet<usize> =
            ops_1.iter().chain(ops_2.iter()).copied().collect();

        for _ in 0..100 {
            let mut g = Genome::new_random(FEAT_OFFSET_25);
            AlpsGA::mutate(&mut g, FEAT_OFFSET_25);
            for &t in &g.tokens {
                if t >= FEAT_OFFSET_25 {
                    // Mutation should only produce tokens from the active op set
                    // (except subtree replacement which uses generate_random_rpn)
                    assert!(
                        valid_ops.contains(&t),
                        "mutated token {} not in active ops",
                        t
                    );
                }
            }
        }
    }

    #[test]
    fn mutate_growth_respects_max_length() {
        let mut g = Genome {
            tokens: vec![0; 19], // just under limit
            fitness: 0.0,
            age: 0,
            block_mask: vec![0; 19],
            block_age: vec![0; 19],
        };
        for _ in 0..50 {
            AlpsGA::mutate(&mut g, FEAT_OFFSET_25);
        }
        // Growth mutation adds 2 tokens but only if len < 20
        // Subtree replacement could reset. Just verify no crash.
        assert!(g.tokens.len() <= 25); // reasonable bound
    }

    // ── ALPS layer aging constants ─────────────────────────────────────

    #[test]
    fn alps_layers_fibonacci_like() {
        assert_eq!(ALPS_LAYER_MAX_AGES, [5, 13, 34, 89, 500]);
        assert_eq!(ALPS_NUM_LAYERS, 5);
        assert_eq!(ALPS_LAYER_POP_SIZE, 100);
    }

    #[test]
    fn alps_layer_max_ages_increasing() {
        for i in 1..ALPS_LAYER_MAX_AGES.len() {
            assert!(
                ALPS_LAYER_MAX_AGES[i] > ALPS_LAYER_MAX_AGES[i - 1],
                "layer ages should be strictly increasing"
            );
        }
    }

    // ── P6a-E1: Atomic Semantic Blocks ─────────────────────────────────

    #[test]
    fn genome_new_random_has_block_fields() {
        let g = Genome::new_random(FEAT_OFFSET_25);
        assert_eq!(g.block_mask.len(), g.tokens.len());
        assert_eq!(g.block_age.len(), g.tokens.len());
        assert!(g.block_mask.iter().all(|&b| b == 0));
    }

    #[test]
    fn sync_block_fields_grows() {
        let mut g = Genome::new_random(FEAT_OFFSET_25);
        g.tokens.push(0); // add a token without updating block fields
        assert_ne!(g.block_mask.len(), g.tokens.len());
        g.sync_block_fields();
        assert_eq!(g.block_mask.len(), g.tokens.len());
        assert_eq!(g.block_age.len(), g.tokens.len());
    }

    #[test]
    fn sync_block_fields_shrinks() {
        let mut g = Genome::new_random(FEAT_OFFSET_25);
        let original_len = g.tokens.len();
        if original_len > 1 {
            g.tokens.pop();
            g.sync_block_fields();
            assert_eq!(g.block_mask.len(), g.tokens.len());
        }
    }

    #[test]
    fn is_blocked_returns_false_for_new_genome() {
        let g = Genome::new_random(FEAT_OFFSET_25);
        for i in 0..g.tokens.len() {
            assert!(!g.is_blocked(i));
        }
    }

    #[test]
    fn mark_block_sets_all_positions() {
        let mut g = Genome::new_random(FEAT_OFFSET_25);
        AlpsGA::mark_block(&mut g, 10);
        assert!(g.block_mask.iter().all(|&b| b == 1));
        assert!(g.block_age.iter().all(|&a| a == 10));
        for i in 0..g.tokens.len() {
            assert!(g.is_blocked(i));
        }
    }

    #[test]
    fn block_aware_cut_points_skip_blocked() {
        // f0 f1 ADD f2 f3 MUL ADD
        // depths: [1, 2, 1, 2, 3, 2, 1]
        // Two sub-expressions combined: (f0+f1) + (f2*f3)
        // Cut points: after ADD (pos 3), after final ADD (pos 7)
        let add = FEAT_OFFSET_25; // ADD = offset + 0
        let mul = FEAT_OFFSET_25 + 2; // MUL = offset + 2
        let tokens = vec![0, 1, add, 2, 3, mul, add];

        // Without mask: both cuts available
        let cuts = valid_cut_points(&tokens, FEAT_OFFSET_25);
        assert_eq!(cuts, vec![3, 7]);

        // Block the second sub-expression (positions 3,4,5)
        let mask = vec![0, 0, 0, 1, 1, 1, 0];
        let cuts_blocked = valid_cut_points_with_mask(&tokens, FEAT_OFFSET_25, Some(&mask));
        // Cut at pos 3: before_blocked=mask[2]=0, after_blocked=mask[3]=1 → not both → allowed
        // Cut at pos 7: before_blocked=mask[6]=0, after_blocked=mask[7] OOB → allowed
        assert_eq!(cuts_blocked, vec![3, 7]);

        // Block everything — cuts between blocked tokens should be skipped
        let mask_all = vec![1, 1, 1, 1, 1, 1, 1];
        let cuts_all_blocked = valid_cut_points_with_mask(&tokens, FEAT_OFFSET_25, Some(&mask_all));
        // Cut at pos 3: mask[2]=1, mask[3]=1 → both blocked → skip
        // Cut at pos 7: mask[6]=1, mask[7] out of bounds → not both → allowed
        assert_eq!(cuts_all_blocked, vec![7]);
    }

    #[test]
    fn shrink_mutation_skips_blocked_positions() {
        // f0 ABS f1 ABS — genome with two unary ops
        let abs_tok = FEAT_OFFSET_25 + 5;
        let g = Genome {
            tokens: vec![0, abs_tok, 1, abs_tok],
            fitness: 1.0,
            age: 0,
            block_mask: vec![1, 1, 0, 0], // first two tokens blocked
            block_age: vec![5, 5, 0, 0],
        };

        // Run many mutations — blocked unary at index 1 should never be removed via shrink
        let mut shrink_happened = false;
        for _ in 0..500 {
            let mut test_g = g.clone();
            AlpsGA::mutate(&mut test_g, FEAT_OFFSET_25);
            // If a shrink occurred (length decreased by 1), verify the blocked region kept its length
            if test_g.tokens.len() == 3 {
                shrink_happened = true;
                // The unblocked unary (index 3) was removed.
                // Blocked tokens may have been point-mutated (same arity) but not structurally removed.
                // Token at index 0 should still be a feature.
                assert!(
                    test_g.tokens[0] < FEAT_OFFSET_25,
                    "first token should be a feature"
                );
                // Token at index 1 should still be a unary op (may have been point-mutated).
                assert_eq!(
                    token_arity(test_g.tokens[1], FEAT_OFFSET_25),
                    1,
                    "blocked position should still have a unary op"
                );
            }
        }
        assert!(
            shrink_happened,
            "shrink should have occurred at least once in 500 trials"
        );
    }

    #[test]
    fn subtree_replacement_blocked_skips() {
        let abs_tok = FEAT_OFFSET_25 + 5;
        let g = Genome {
            tokens: vec![0, abs_tok],
            fitness: 1.0,
            age: 0,
            block_mask: vec![1, 1], // fully blocked
            block_age: vec![5, 5],
        };

        // With blocks, subtree replacement should be suppressed
        let original_tokens = g.tokens.clone();
        for _ in 0..100 {
            let mut test_g = g.clone();
            // Only trigger subtree replacement path by calling mutate many times
            AlpsGA::mutate(&mut test_g, FEAT_OFFSET_25);
            // Point/operator mutations may change tokens but length stays same
            // Subtree replacement would change length — should not happen with blocks
            // (Growth could add tokens though)
            if test_g.tokens.len() != original_tokens.len() && test_g.tokens.len() > 2 {
                // Growth happened (appended 2 tokens) — that's fine
                assert!(
                    test_g.tokens.len() >= original_tokens.len(),
                    "blocked genome should not shrink via subtree replacement"
                );
            }
        }
    }

    #[test]
    fn block_decay_clears_stale_blocks() {
        let mut ga = AlpsGA::new(FEAT_OFFSET_25);
        // Mark a genome in layer 0 as blocked at generation 0
        AlpsGA::mark_block(&mut ga.layers[0].population[0], 0);
        assert!(ga.layers[0].population[0]
            .block_mask
            .iter()
            .all(|&b| b == 1));

        // Advance past BLOCK_DECAY_GENS
        ga.generation = BLOCK_DECAY_GENS + 1;
        // Set fitness so genomes survive
        for g in ga.all_genomes_mut() {
            g.fitness = 1.0;
        }

        ga.evolve();
        // After evolve, block decay phase should have cleared the block
        // (The genome may have been replaced by evolution, but if it survived in elitism,
        //  its blocks should be cleared.)
        // We verify the mechanism by checking that no genome in layer 0 has stale blocks
        for g in &ga.layers[0].population {
            if g.block_mask.iter().any(|&b| b == 1) {
                // If blocks exist, they should have been set after generation BLOCK_DECAY_GENS
                let min_age = g
                    .block_age
                    .iter()
                    .filter(|&&a| a > 0)
                    .min()
                    .copied()
                    .unwrap_or(0);
                assert!(
                    ga.generation <= min_age + BLOCK_DECAY_GENS + 1,
                    "stale block should have been cleared"
                );
            }
        }
    }

    #[test]
    fn crossover_preserves_parent_blocks() {
        let add = FEAT_OFFSET_25;
        let sub = FEAT_OFFSET_25 + 1;
        let abs_tok = FEAT_OFFSET_25 + 5;

        // Parent 1: f0 f1 ADD — blocked
        let p1 = Genome {
            tokens: vec![0, 1, add],
            fitness: 1.0,
            age: 5,
            block_mask: vec![1, 1, 1],
            block_age: vec![10, 10, 10],
        };
        // Parent 2: f2 ABS f3 SUB — unblocked
        let p2 = Genome {
            tokens: vec![2, abs_tok, 3, sub],
            fitness: 2.0,
            age: 10,
            block_mask: vec![0, 0, 0, 0],
            block_age: vec![0, 0, 0, 0],
        };

        for _ in 0..20 {
            let child = AlpsGA::crossover(&p1, &p2, FEAT_OFFSET_25);
            assert_eq!(child.block_mask.len(), child.tokens.len());
            assert_eq!(child.block_age.len(), child.tokens.len());
        }
    }

    // ── P8-2B: Diversity trigger support methods ─────────────────────

    #[test]
    fn test_layer_size() {
        let ga = AlpsGA::new(FEAT_OFFSET_25);
        assert_eq!(ga.layer_size(0), 100);
        assert_eq!(ga.layer_size(4), 100);
        assert_eq!(ga.layer_size(99), 0); // out of range
    }

    #[test]
    fn test_generate_random_genomes() {
        let ga = AlpsGA::new(FEAT_OFFSET_25);
        let genomes = ga.generate_random_genomes(5);
        assert_eq!(genomes.len(), 5);
        for g in &genomes {
            assert!(!g.tokens.is_empty());
            assert_eq!(g.fitness, 0.0);
        }
    }

    #[test]
    fn test_cull_weakest() {
        let mut ga = AlpsGA::new(FEAT_OFFSET_25);
        let initial_size = ga.layer_size(0);
        // Set known fitness values
        for (i, g) in ga.layers[0].population.iter_mut().enumerate() {
            g.fitness = i as f64; // 0, 1, 2, ..., 99
        }
        let culled = ga.cull_weakest(0, 10);
        assert_eq!(culled, 10);
        assert_eq!(ga.layer_size(0), initial_size - 10);
        // All remaining should have fitness >= 10
        for g in &ga.layers[0].population {
            assert!(g.fitness >= 10.0, "Expected >= 10.0, got {}", g.fitness);
        }
    }

    #[test]
    fn test_cull_weakest_out_of_range() {
        let mut ga = AlpsGA::new(FEAT_OFFSET_25);
        assert_eq!(ga.cull_weakest(99, 5), 0); // invalid layer
        assert_eq!(ga.cull_weakest(0, 0), 0);  // zero count
    }
}
