//! RPN formula state for MCTS.
//!
//! States represent partial RPN formulas. Actions are legal next tokens.
//! A state is terminal when stack_depth == 1 (complete formula).

/// RPN action space: possible next tokens.
#[derive(Debug, Clone)]
pub struct ActionSpace {
    /// Feature token indices: 0..feat_offset
    pub feat_offset: usize,
    /// Total vocabulary size: feat_offset + n_operators
    #[allow(dead_code)] // P8: used in LLM prior policy token distribution
    pub vocab_size: usize,
    /// Arity of each operator (index = token - feat_offset): 1=unary, 2=binary
    pub operator_arities: Vec<u32>,
    /// Maximum formula length
    pub max_length: usize,
}

impl ActionSpace {
    /// Create action space from factor config.
    ///
    /// Active operators (14 of 23):
    /// - Unary (9):  ABS, SIGN, DELAY1, DELAY5, TS_MEAN, TS_STD, TS_RANK, TS_MIN, TS_MAX
    /// - Binary (5): ADD, SUB, MUL, DIV, TS_CORR
    pub fn new(feat_offset: usize) -> Self {
        // 23 operators total (VM retains all for backward compat)
        // Arities: 0-4 are binary (ADD,SUB,MUL,DIV,POW), 5-15 are unary,
        //          16 is binary (TS_CORR), 17-22 are unary
        let mut arities = Vec::with_capacity(23);
        for op in 0..23 {
            let arity = match op {
                0..=4 => 2,   // ADD, SUB, MUL, DIV, POW
                5..=15 => 1, // ABS, SIGN, LOG, EXP, DELAY1, DELAY5, TS_MEAN, TS_STD, TS_RANK, NEG, SQRT
                16 => 2,     // TS_CORR
                17..=22 => 1, // TS_MIN, TS_MAX, TS_SKEW, TS_KURT, SIGMOID, TANH
                _ => 1,
            };
            arities.push(arity);
        }

        Self {
            feat_offset,
            vocab_size: feat_offset + 23,
            operator_arities: arities,
            max_length: 20,
        }
    }

    /// Get legal actions given current stack depth and formula length.
    ///
    /// Rules:
    /// - Features (push): always legal if formula not at max length
    /// - Unary operators (pop 1, push 1): legal if stack >= 1
    /// - Binary operators (pop 2, push 1): legal if stack >= 2
    /// - Must be able to reach terminal (stack=1) within remaining tokens
    pub fn legal_actions(&self, stack_depth: u32, current_length: usize) -> Vec<u32> {
        let remaining = self.max_length - current_length;
        if remaining == 0 {
            return Vec::new();
        }

        let mut actions = Vec::new();

        // Features (push): stack grows by 1
        // Only if we can reduce stack to 1 within remaining-1 tokens
        // Each binary op reduces stack by 1, so we need (stack_depth + 1 - 1) = stack_depth
        // binary ops within remaining-1 tokens
        if stack_depth < remaining as u32 {
            for token in 0..self.feat_offset {
                actions.push(token as u32);
            }
        }

        // Operators
        for (op_idx, &arity) in self.operator_arities.iter().enumerate() {
            let token = (self.feat_offset + op_idx) as u32;

            if arity == 1 && stack_depth >= 1 {
                // Unary: stack unchanged, always legal if stack >= 1
                actions.push(token);
            } else if arity == 2 && stack_depth >= 2 {
                // Binary: stack reduces by 1
                actions.push(token);
            }
        }

        actions
    }

    /// Compute new stack depth after applying an action.
    pub fn stack_after_action(&self, stack_depth: u32, action: u32) -> u32 {
        let action = action as usize;
        if action < self.feat_offset {
            // Feature: push onto stack
            stack_depth + 1
        } else {
            let op_idx = action - self.feat_offset;
            let arity = self.operator_arities[op_idx];
            // Operator: pop `arity` and push 1
            stack_depth - arity + 1
        }
    }

    /// Check if a state is terminal (complete formula).
    pub fn is_terminal(&self, stack_depth: u32, length: usize) -> bool {
        stack_depth == 1 && length > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_space_creation() {
        let space = ActionSpace::new(25);
        assert_eq!(space.feat_offset, 25);
        assert_eq!(space.vocab_size, 48);
        assert_eq!(space.operator_arities.len(), 23);
    }

    #[test]
    fn test_legal_actions_empty_stack() {
        let space = ActionSpace::new(3);
        let actions = space.legal_actions(0, 0);
        // Only features are legal (push onto empty stack)
        assert!(!actions.is_empty());
        assert!(actions.iter().all(|&a| (a as usize) < space.feat_offset));
    }

    #[test]
    fn test_legal_actions_stack_depth_1() {
        let space = ActionSpace::new(3);
        let actions = space.legal_actions(1, 1);
        // Features (push) and unary operators are legal
        // Binary operators need stack >= 2
        let has_features = actions.iter().any(|&a| (a as usize) < space.feat_offset);
        let has_unary = actions.iter().any(|&a| {
            let idx = a as usize;
            idx >= space.feat_offset && space.operator_arities[idx - space.feat_offset] == 1
        });
        assert!(has_features);
        assert!(has_unary);
    }

    #[test]
    fn test_legal_actions_stack_depth_2() {
        let space = ActionSpace::new(3);
        let actions = space.legal_actions(2, 2);
        // Binary operators should now be legal
        let has_binary = actions.iter().any(|&a| {
            let idx = a as usize;
            idx >= space.feat_offset && space.operator_arities[idx - space.feat_offset] == 2
        });
        assert!(has_binary);
    }

    #[test]
    fn test_stack_after_feature() {
        let space = ActionSpace::new(25);
        assert_eq!(space.stack_after_action(0, 0), 1); // push feature
        assert_eq!(space.stack_after_action(2, 5), 3); // push another feature
    }

    #[test]
    fn test_stack_after_operator() {
        let space = ActionSpace::new(25);
        // Unary (e.g., ABS at feat_offset+5): stack unchanged
        assert_eq!(space.stack_after_action(2, 30), 2);
        // Binary (e.g., ADD at feat_offset+0): stack reduces by 1
        assert_eq!(space.stack_after_action(2, 25), 1);
    }

    #[test]
    fn test_terminal_state() {
        let space = ActionSpace::new(25);
        assert!(!space.is_terminal(0, 0)); // empty
        assert!(!space.is_terminal(2, 3)); // stack > 1
        assert!(space.is_terminal(1, 3)); // stack = 1, has tokens
    }
}
