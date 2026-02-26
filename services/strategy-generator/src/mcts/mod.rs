//! P6-4A: LLM-Prior MCTS Symbolic Regression.
//!
//! Monte Carlo Tree Search for discovering high-fitness RPN formulas.
//! Uses arena allocation (contiguous `Vec<Node>` + `u32` indices) for
//! cache-coherent tree traversal and zero-cost GC (drop entire arena).
//!
//! Integration: MCTS injects seed formulas into ALPS Layer 0 via `inject_genomes`.

pub mod arena;
pub mod policy;
pub mod search;
pub mod state;
