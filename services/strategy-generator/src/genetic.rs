use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genome {
    pub tokens: Vec<usize>,
    pub fitness: f64,
    /// Number of generations this genome has survived (used by ALPS).
    #[serde(default)]
    pub age: usize,
}

impl Genome {
    pub fn new_random(feat_offset: usize) -> Self {
        Self {
            tokens: generate_random_rpn(5, feat_offset),
            fitness: 0.0,
            age: 0,
        }
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
const ALPS_LAYER_MAX_AGES: [usize; 5] = [5, 13, 34, 89, usize::MAX];
const ALPS_LAYER_POP_SIZE: usize = 100;
const ALPS_NUM_LAYERS: usize = 5;

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
                genome.tokens = generate_random_rpn(5, feat_offset);
                genome.fitness = 0.0;
                genome.age = 0;
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

    /// Number of promotions that occurred in the last evolve() call.
    /// (Stored for logging; reset each generation.)
    pub fn evolve(&mut self) -> usize {
        let mut rng = rand::thread_rng();
        let mut total_promotions = 0_usize;

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
            let mut promoted = Vec::new();
            self.layers[layer_idx].population.retain(|g| {
                if g.age > max_age {
                    promoted.push(g.clone());
                    false
                } else {
                    true
                }
            });

            // Try to insert promoted genomes into the next layer
            let next_layer = &mut self.layers[layer_idx + 1];
            for genome in promoted {
                if next_layer.population.len() < ALPS_LAYER_POP_SIZE {
                    // Room available — just insert
                    next_layer.population.push(genome);
                    total_promotions += 1;
                } else {
                    // Replace worst genome if promoted genome is fitter
                    next_layer.sort_by_fitness();
                    if let Some(worst) = next_layer.population.last() {
                        if genome.fitness > worst.fitness {
                            next_layer.population.pop();
                            next_layer.population.push(genome);
                            total_promotions += 1;
                        }
                        // else: discard (not fit enough for next layer)
                    }
                }
            }
        }

        // Phase 3: Replenish layer 0 with fresh random genomes
        let layer0 = &mut self.layers[0];
        while layer0.population.len() < ALPS_LAYER_POP_SIZE {
            layer0.population.push(Genome::new_random(self.feat_offset));
        }

        // Phase 4: Evolve each layer independently
        for layer in &mut self.layers {
            if layer.population.is_empty() {
                continue;
            }

            layer.sort_by_fitness();
            layer.deduplicate(self.feat_offset);

            let pop_size = layer.population.len();
            let elitism_count = (pop_size as f64 * 0.05).max(2.0).min(pop_size as f64) as usize;

            let mut new_pop = Vec::with_capacity(pop_size);

            // Elitism: keep top genomes
            for i in 0..elitism_count.min(pop_size) {
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
                    Self::mutate(&mut child, self.feat_offset);
                    new_pop.push(child);
                } else if r < 0.75 {
                    let mut child = parent1.clone();
                    Self::mutate(&mut child, self.feat_offset);
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
        total_promotions
    }

    /// Single-point crossover: take prefix of parent1, suffix of parent2.
    fn crossover(parent1: &Genome, parent2: &Genome, feat_offset: usize) -> Genome {
        let mut rng = rand::thread_rng();

        let cuts1 = valid_cut_points(&parent1.tokens, feat_offset);
        let cuts2 = valid_cut_points(&parent2.tokens, feat_offset);

        if cuts1.is_empty() || cuts2.is_empty() {
            return parent1.clone();
        }

        let cut1 = cuts1[rng.gen_range(0..cuts1.len())];
        let cut2 = cuts2[rng.gen_range(0..cuts2.len())];

        let mut tokens = parent1.tokens[..cut1].to_vec();
        tokens.extend_from_slice(&parent2.tokens[cut2..]);

        if tokens.len() > 20 {
            tokens.truncate(20);
        }

        Genome {
            tokens,
            fitness: 0.0,
            age: 0,
        }
    }

    /// Mutation operators (no stagnation-dependent rates — ALPS handles diversity).
    fn mutate(genome: &mut Genome, feat_offset: usize) {
        let mut rng = rand::thread_rng();
        let (ops_1, ops_2) = build_ops(feat_offset);

        // 1. Point mutation (40%): change any token to same-arity token
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
        if rng.gen_bool(0.08) && genome.tokens.len() < 20 {
            let feat = rng.gen_range(0..feat_offset);
            let op = ops_2[rng.gen_range(0..ops_2.len())];
            genome.tokens.push(feat);
            genome.tokens.push(op);
        }

        // 4. Shrink mutation (20%): remove a unary op (stack-neutral)
        if rng.gen_bool(0.20) && genome.tokens.len() > 3 {
            let unary_indices: Vec<usize> = genome
                .tokens
                .iter()
                .enumerate()
                .filter(|(_, &t)| t >= feat_offset && token_arity(t, feat_offset) == 1)
                .map(|(i, _)| i)
                .collect();
            if !unary_indices.is_empty() {
                let idx = unary_indices[rng.gen_range(0..unary_indices.len())];
                genome.tokens.remove(idx);
            }
        }

        // 5. Subtree replacement (10%): replace genome with new random subtree
        if rng.gen_bool(0.10) {
            let new_subtree = generate_random_rpn(5, feat_offset);
            genome.tokens = new_subtree;
            genome.age = 0; // Reset age on full replacement
        }
    }
}

/// Find positions in the token sequence where the running stack depth is exactly 1.
/// These are valid "cut points" for crossover — the formula up to that point
/// produces exactly one value on the stack.
fn valid_cut_points(tokens: &[usize], feat_offset: usize) -> Vec<usize> {
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
            cuts.push(i + 1); // cut AFTER this token
        }
    }

    cuts
}
