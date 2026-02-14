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

fn generate_random_rpn(max_depth: usize, feat_offset: usize) -> Vec<usize> {
    let mut rng = rand::thread_rng();

    // Features: 0..feat_offset
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

        // A. Add Feature (Grow)
        if stack_depth < max_depth && steps < target_len {
            choices.push("FEAT");
        }

        // B. Unary Op (Mutation/Transform)
        if stack_depth >= 1 {
            choices.push("OP1");
        }

        // C. Binary Op (Collapse)
        if stack_depth >= 2 {
            choices.push("OP2");
        }

        // D. Ternary Op (Collapse)
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

        // Elitism: Keep top 10%
        let elitism_count = (pop_size as f64 * 0.1) as usize;
        let mut new_pop = Vec::with_capacity(pop_size);

        for i in 0..elitism_count {
            new_pop.push(self.population[i].clone());
        }

        // Offspring
        while new_pop.len() < pop_size {
            // Tournament selection
            let parent1 = self.tournament_select();
            let parent2 = self.tournament_select();

            if rng.gen_bool(0.5) {
                let mut child = parent1.clone();
                Self::mutate(&mut child, self.feat_offset);
                new_pop.push(child);
            } else {
                let mut child = parent2.clone();
                Self::mutate(&mut child, self.feat_offset);
                new_pop.push(child);
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

    fn mutate(genome: &mut Genome, feat_offset: usize) {
        let mut rng = rand::thread_rng();
        // 1. Point mutation on a feature token
        if !genome.tokens.is_empty() {
            let idx = rng.gen_range(0..genome.tokens.len());
            let old = genome.tokens[idx];

            if old < feat_offset {
                // Mutate feature to another feature
                genome.tokens[idx] = rng.gen_range(0..feat_offset);
            }
        }

        // 2. Growth mutation: append Feature + BinaryOp (stack-neutral)
        if rng.gen_bool(0.1) {
            let feat = rng.gen_range(0..feat_offset);
            let (_, ops_2, _) = build_ops(feat_offset);
            let op = ops_2[rng.gen_range(0..ops_2.len())];

            genome.tokens.push(feat);
            genome.tokens.push(op);
        }
    }
}
