pub mod engine;
pub mod ops;

#[allow(clippy::module_inception)]
pub mod vm; // The core stack VM 

// Re-export core items
pub use vm::StackVM;
