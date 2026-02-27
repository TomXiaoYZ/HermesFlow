#![allow(dead_code)] // P6-4A: not yet integrated into ALPS evolution loop
//! Arena allocator for MCTS tree nodes.
//!
//! All nodes live in a contiguous `Vec<Node>`. Parent/child relationships
//! use `u32` indices instead of `Rc<RefCell<T>>` for:
//! - Cache coherence: sequential memory layout
//! - Zero-cost GC: drop the entire `Vec` after each search round
//! - No reference counting overhead

/// Sentinel value for "no node" (null pointer equivalent).
pub const NULL_NODE: u32 = u32::MAX;

/// A single MCTS tree node stored in the arena.
#[derive(Debug, Clone)]
pub struct Node {
    /// Parent node index (NULL_NODE for root)
    pub parent: u32,
    /// Indices of child nodes in the arena
    pub children: Vec<u32>,
    /// Action that led to this node (token index in RPN vocabulary)
    pub action: u32,
    /// Number of times this node has been visited
    pub visit_count: u32,
    /// Sum of rewards from all rollouts through this node
    pub total_reward: f64,
    /// Maximum reward seen in subtree (for extreme bandit PUCT)
    pub max_reward: f64,
    /// Prior probability from policy (LLM or uniform)
    pub prior: f64,
    /// Whether this is a terminal state (complete RPN formula)
    pub is_terminal: bool,
    /// Stack depth at this node (for RPN validity checking)
    pub stack_depth: u32,
}

impl Node {
    pub fn mean_reward(&self) -> f64 {
        if self.visit_count == 0 {
            0.0
        } else {
            self.total_reward / self.visit_count as f64
        }
    }
}

/// Arena-based tree allocator.
///
/// Nodes are allocated by pushing to a `Vec`. Node references are `u32`
/// indices into this vector. The entire arena can be dropped at once
/// after a search round completes.
pub struct Arena {
    nodes: Vec<Node>,
}

impl Arena {
    /// Create a new arena with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
        }
    }

    /// Allocate a new node and return its index.
    pub fn alloc(&mut self, node: Node) -> u32 {
        let idx = self.nodes.len() as u32;
        self.nodes.push(node);
        idx
    }

    /// Get an immutable reference to a node by index.
    pub fn get(&self, idx: u32) -> &Node {
        &self.nodes[idx as usize]
    }

    /// Get a mutable reference to a node by index.
    pub fn get_mut(&mut self, idx: u32) -> &mut Node {
        &mut self.nodes[idx as usize]
    }

    /// Number of nodes in the arena.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the arena is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Create root node and return its index.
    pub fn create_root() -> (Self, u32) {
        let mut arena = Self::with_capacity(1024);
        let root = arena.alloc(Node {
            parent: NULL_NODE,
            children: Vec::new(),
            action: 0,
            visit_count: 0,
            total_reward: 0.0,
            max_reward: f64::NEG_INFINITY,
            prior: 1.0,
            is_terminal: false,
            stack_depth: 0,
        });
        (arena, root)
    }

    /// Add a child to a parent node with the given action and prior.
    pub fn add_child(
        &mut self,
        parent_idx: u32,
        action: u32,
        prior: f64,
        stack_depth: u32,
        is_terminal: bool,
    ) -> u32 {
        let child_idx = self.alloc(Node {
            parent: parent_idx,
            children: Vec::new(),
            action,
            visit_count: 0,
            total_reward: 0.0,
            max_reward: f64::NEG_INFINITY,
            prior,
            is_terminal,
            stack_depth,
        });
        self.nodes[parent_idx as usize].children.push(child_idx);
        child_idx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_creation() {
        let (arena, root) = Arena::create_root();
        assert_eq!(root, 0);
        assert_eq!(arena.len(), 1);
        assert_eq!(arena.get(root).parent, NULL_NODE);
        assert_eq!(arena.get(root).stack_depth, 0);
    }

    #[test]
    fn test_add_children() {
        let (mut arena, root) = Arena::create_root();
        let child1 = arena.add_child(root, 5, 0.3, 1, false);
        let child2 = arena.add_child(root, 10, 0.7, 1, false);

        assert_eq!(arena.len(), 3);
        assert_eq!(arena.get(root).children.len(), 2);
        assert_eq!(arena.get(child1).parent, root);
        assert_eq!(arena.get(child2).parent, root);
        assert_eq!(arena.get(child1).action, 5);
        assert_eq!(arena.get(child2).action, 10);
    }

    #[test]
    fn test_node_statistics() {
        let (mut arena, root) = Arena::create_root();
        let node = arena.get_mut(root);
        node.visit_count = 10;
        node.total_reward = 5.0;
        assert!((node.mean_reward() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_null_node_sentinel() {
        assert_eq!(NULL_NODE, u32::MAX);
    }
}
