use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genome {
    pub tokens: Vec<usize>,
    pub fitness: f64,
}

impl Genome {
    pub fn new_random(feat_offset: usize) -> Self {
        Self {
            tokens: generate_random_rpn(5, feat_offset),
            fitness: 0.0,
        }
    }
}

/// Build operator token vectors dynamically from feat_offset.
/// Op indices match the StackVM dispatch (vm.rs):
///   Unary:   4(NEG), 5(ABS), 6(SIGN), 8(JUMP), 9(DECAY), 10(DELAY),
///            11(MAX3), 12(TS_MEAN), 13(TS_STD), 14(TS_RANK), 15(TS_SUM),
///            17(CS_RANK), 18(CS_MEAN), 19(LOG), 20(SQRT), 21(INV), 22(TS_DELTA)
///   Binary:  0(ADD), 1(SUB), 2(MUL), 3(DIV), 16(TS_CORR)
///   Ternary: 7(GATE)
fn build_ops(feat_offset: usize) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
    let unary_op_indices: Vec<usize> = vec![
        4, 5, 6, 8, 9, 10, 11, 12, 13, 14, 15, 17, 18, 19, 20, 21, 22,
    ];
    let binary_op_indices: Vec<usize> = vec![0, 1, 2, 3, 16];
    let ternary_op_indices: Vec<usize> = vec![7];

    let ops_1: Vec<usize> = unary_op_indices
        .into_iter()
        .map(|idx| idx + feat_offset)
        .collect();
    let ops_2: Vec<usize> = binary_op_indices
        .into_iter()
        .map(|idx| idx + feat_offset)
        .collect();
    let ops_3: Vec<usize> = ternary_op_indices
        .into_iter()
        .map(|idx| idx + feat_offset)
        .collect();

    (ops_1, ops_2, ops_3)
}

/// Classify a token as feature, unary, binary, or ternary operator.
fn token_arity(token: usize, feat_offset: usize) -> usize {
    if token < feat_offset {
        return 0; // feature (pushes to stack)
    }
    let op_idx = token - feat_offset;
    match op_idx {
        0..=3 | 16 => 2,  // binary: ADD, SUB, MUL, DIV, TS_CORR
        7 => 3,           // ternary: GATE
        _ => 1,           // unary: everything else
    }
}

fn generate_random_rpn(max_depth: usize, feat_offset: usize) -> Vec<usize> {
    let mut rng = rand::thread_rng();

    let features: Vec<usize> = (0..feat_offset).collect();
    let (ops_1, ops_2, ops_3) = build_ops(feat_offset);

    let mut tokens = Vec::new();

    // Step 1: Push at least one feature
    tokens.push(features[rng.gen_range(0..features.len())]);
    let mut stack_depth = 1;

    let mut steps = 0;
    let target_len = rng.gen_range(3..15);

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
        if stack_depth >= 3 {
            choices.push("OP3");
        }

        // Forced collapse if length exceeded
        if steps >= target_len {
            choices.clear();
            if stack_depth >= 3 {
                choices.push("OP3");
                choices.push("OP2");
            } else if stack_depth >= 2 {
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
            "OP3" => {
                tokens.push(ops_3[rng.gen_range(0..ops_3.len())]);
                stack_depth -= 2;
            }
            _ => {}
        }

        steps += 1;
    }

    tokens
}

pub struct GeneticAlgorithm {
    pub population: Vec<Genome>,
    pub generation: usize,
    pub best_genome: Option<Genome>,
    pub feat_offset: usize,
}

impl GeneticAlgorithm {
    pub fn new(pop_size: usize, feat_offset: usize) -> Self {
        let mut population = Vec::with_capacity(pop_size);
        for _ in 0..pop_size {
            population.push(Genome::new_random(feat_offset));
        }

        Self {
            population,
            generation: 0,
            best_genome: None,
            feat_offset,
        }
    }

    pub fn evolve(&mut self) {
        let mut rng = rand::thread_rng();
        let pop_size = self.population.len();

        // Sort by fitness DESC
        self.population.sort_by(|a, b| {
            b.fitness
                .partial_cmp(&a.fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Update Best
        if let Some(best) = self.population.first() {
            if let Some(current_best) = &self.best_genome {
                if best.fitness > current_best.fitness {
                    self.best_genome = Some(best.clone());
                }
            } else {
                self.best_genome = Some(best.clone());
            }
        }

        // Elitism: Keep top 5% (min 2)
        let elitism_count = (pop_size as f64 * 0.05).max(2.0) as usize;
        let mut new_pop = Vec::with_capacity(pop_size);

        for i in 0..elitism_count {
            new_pop.push(self.population[i].clone());
        }

        // Fill rest with offspring
        while new_pop.len() < pop_size {
            let parent1 = self.tournament_select();
            let parent2 = self.tournament_select();

            let r: f64 = rng.gen();
            if r < 0.35 {
                // 35%: Crossover + mutate
                let mut child = Self::crossover(parent1, parent2, self.feat_offset);
                Self::mutate(&mut child, self.feat_offset);
                new_pop.push(child);
            } else if r < 0.75 {
                // 40%: Clone parent + mutate
                let mut child = parent1.clone();
                Self::mutate(&mut child, self.feat_offset);
                new_pop.push(child);
            } else {
                // 25%: Fresh random genome (immigration)
                new_pop.push(Genome::new_random(self.feat_offset));
            }
        }

        self.population = new_pop;
        self.generation += 1;
    }

    fn tournament_select(&self) -> &Genome {
        let mut rng = rand::thread_rng();
        let k = 3;
        let mut best: Option<&Genome> = None;

        for _ in 0..k {
            let idx = rng.gen_range(0..self.population.len());
            let candidate = &self.population[idx];
            if best.is_none() || candidate.fitness > best.unwrap().fitness {
                best = Some(candidate);
            }
        }
        best.unwrap()
    }

    /// Single-point crossover: take prefix of parent1, suffix of parent2.
    /// Finds valid cut points where both parents have stack_depth == 1.
    fn crossover(parent1: &Genome, parent2: &Genome, feat_offset: usize) -> Genome {
        let mut rng = rand::thread_rng();

        // Find positions where stack depth == 1 (valid split points)
        let cuts1 = valid_cut_points(&parent1.tokens, feat_offset);
        let cuts2 = valid_cut_points(&parent2.tokens, feat_offset);

        if cuts1.is_empty() || cuts2.is_empty() {
            // Can't find valid cuts; return mutated clone of parent1
            return parent1.clone();
        }

        let cut1 = cuts1[rng.gen_range(0..cuts1.len())];
        let cut2 = cuts2[rng.gen_range(0..cuts2.len())];

        // Take parent1[..cut1] + parent2[cut2..]
        let mut tokens = parent1.tokens[..cut1].to_vec();
        tokens.extend_from_slice(&parent2.tokens[cut2..]);

        // Cap length to avoid bloat
        if tokens.len() > 30 {
            tokens.truncate(30);
        }

        Genome {
            tokens,
            fitness: 0.0,
        }
    }

    fn mutate(genome: &mut Genome, feat_offset: usize) {
        let mut rng = rand::thread_rng();
        let (ops_1, ops_2, ops_3) = build_ops(feat_offset);

        // 1. Point mutation (40% chance): change any token to same-arity token
        if rng.gen_bool(0.4) && !genome.tokens.is_empty() {
            let idx = rng.gen_range(0..genome.tokens.len());
            let old = genome.tokens[idx];

            if old < feat_offset {
                // Feature → another feature
                genome.tokens[idx] = rng.gen_range(0..feat_offset);
            } else {
                // Operator → another operator of same arity
                let arity = token_arity(old, feat_offset);
                let pool = match arity {
                    1 => &ops_1,
                    2 => &ops_2,
                    3 => &ops_3,
                    _ => &ops_1,
                };
                genome.tokens[idx] = pool[rng.gen_range(0..pool.len())];
            }
        }

        // 2. Operator mutation (20% chance): swap a random operator for same-arity
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
                    3 => &ops_3,
                    _ => &ops_1,
                };
                genome.tokens[idx] = pool[rng.gen_range(0..pool.len())];
            }
        }

        // 3. Growth mutation (10% chance): append Feature + BinaryOp (stack-neutral)
        if rng.gen_bool(0.1) && genome.tokens.len() < 30 {
            let feat = rng.gen_range(0..feat_offset);
            let op = ops_2[rng.gen_range(0..ops_2.len())];
            genome.tokens.push(feat);
            genome.tokens.push(op);
        }

        // 4. Shrink mutation (10% chance): remove a unary op (stack-neutral)
        if rng.gen_bool(0.1) && genome.tokens.len() > 3 {
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

        // 5. Subtree replacement (10% chance): replace genome with new random subtree
        if rng.gen_bool(0.1) {
            let new_subtree = generate_random_rpn(5, feat_offset);
            genome.tokens = new_subtree;
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
