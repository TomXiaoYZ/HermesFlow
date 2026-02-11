use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genome {
    pub tokens: Vec<usize>,
    pub fitness: f64,
}

impl Genome {
    pub fn new_random() -> Self {
        Self {
            tokens: generate_random_rpn(5),
            fitness: 0.0,
        }
    }
}

// Configuration matching Backtest Engine
// Features: 0-13 (14 dims)
// Offset: 14
// Ops: 0-18 (19 ops)

const FEAT_OFFSET: usize = 14;

fn generate_random_rpn(max_depth: usize) -> Vec<usize> {
    let mut rng = rand::thread_rng();

    // Features 0-13
    let features: Vec<usize> = (0..14).collect();

    // Ops by Arity
    // Offset 14.
    // Arity 1: NEG(4), ABS(5), SIGN(6), JUMP(8), DECAY(9), DELAY(10), MAX3(11)
    //          TS_MEAN(12), TS_STD(13), TS_RANK(14), TS_SUM(15), CS_RANK(17), CS_MEAN(18)
    //          New: LOG(19), SQRT(20), INV(21), TS_DELTA(22)
    let ops_1: Vec<usize> = vec![
        18, 19, 20, 22, 23, 24,
        25, // Basic (Note: indices in genetic.rs were: 4->18, 5->19... wait, offset=14)
        // vm.rs dispatch: 4=NEG. But in RPN token = 4 + 14 = 18.
        // So existing 18=NEG, 19=ABS, 20=SIGN.
        // 8=JUMP -> 8+14=22. 9=DECAY -> 23. 10=DELAY -> 24. 11=MAX3 -> 25.
        // 12=TS_MEAN -> 26. 13=TS_STD -> 27. 14=TS_RANK -> 28. 15=TS_SUM -> 29.
        // 17=CS_RANK -> 31. 18=CS_MEAN -> 32.

        // New Ops indices in VM:
        // 19=LOG -> 19+14=33
        // 20=SQRT -> 20+14=34
        // 21=INV -> 21+14=35
        // 22=TS_DELTA -> 22+14=36
        18, 19, 20, 22, 23, 24, 25, // Basic: NEG..MAX3
        26, 27, 28, 29, 31, 32, // TS/CS ops
        33, 34, 35, 36, // New: LOG, SQRT, INV, DELAY
    ];

    // Arity 2: ADD(0), SUB(1), MUL(2), DIV(3), TS_CORR(16)
    let ops_2: Vec<usize> = vec![14, 15, 16, 17, 30];

    // Arity 3: GATE(7)
    let ops_3: Vec<usize> = vec![21];

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
                choices.push("OP1"); // Can't reduce depth 1. Just mutate or break?
                                     // If depth is 1, we are done. Be handled below.
            }

            if stack_depth == 1 {
                break;
            }
        }

        if choices.is_empty() {
            break;
        }

        // Fix: Use slice for choose
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
}

impl GeneticAlgorithm {
    pub fn new(pop_size: usize) -> Self {
        let mut population = Vec::with_capacity(pop_size);
        for _ in 0..pop_size {
            population.push(Genome::new_random());
        }

        Self {
            population,
            generation: 0,
            best_genome: None,
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

            // Crossover logic (simplified: random choice)
            // Or just mutate parent1?
            // For MVP: 50% Crossover, 50% Mutation
            if rng.gen_bool(0.5) {
                // Crossover
                // Single point? RPN crossover is hard because validity.
                // Safer: Just pick one parent and Clone + Mutate heavily.
                // Or specialized RPN crossover (swap subtrees). Implementation is complex.
                // Fallback: Clone parent and Mutate.
                let mut child = parent1.clone();
                Self::mutate(&mut child);
                new_pop.push(child);
            } else {
                // Clone + Mutate.
                let mut child = parent2.clone();
                Self::mutate(&mut child);
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

    fn mutate(genome: &mut Genome) {
        let mut rng = rand::thread_rng();
        // 1. Change a token (Point mutation)
        if !genome.tokens.is_empty() {
            let idx = rng.gen_range(0..genome.tokens.len());
            let old = genome.tokens[idx];

            if old < FEAT_OFFSET {
                // Mutate feature to another feature
                genome.tokens[idx] = rng.gen_range(0..FEAT_OFFSET);
            }
            // Mutating Op is risky unless same arity.
            // Simplified: Just mutate features for now.
        }

        // 2. Append new random logic? (Growth)
        if rng.gen_bool(0.1) {
            // Add: Feature + BinaryOp (maintain stack)
            // Push Feat(1), Op(pop 2 push 1) -> Net -1? No.
            // Stack: [Old] -> [Old, Feat] -> [Op(Old, Feat)]
            // Net effect: 1 in -> 1 out.
            // So we append: FEAT, OP2.
            let feat = rng.gen_range(0..FEAT_OFFSET);
            let ops_2: Vec<usize> = vec![14, 15, 16, 17, 30]; // Same as above
            let op = ops_2[rng.gen_range(0..ops_2.len())];

            genome.tokens.push(feat);
            genome.tokens.push(op);
        }
    }
}
