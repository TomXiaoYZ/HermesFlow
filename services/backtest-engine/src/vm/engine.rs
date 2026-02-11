use crate::vm::StackVM;

pub struct ExecutionEngine {
    pub vm: StackVM,
}

impl Default for ExecutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionEngine {
    pub fn new() -> Self {
        Self { vm: StackVM::new() }
    }
}
