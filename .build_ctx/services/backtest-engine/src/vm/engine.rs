use crate::vm::StackVM;

pub struct ExecutionEngine {
    pub vm: StackVM,
}

impl ExecutionEngine {
    pub fn new() -> Self {
        Self {
            vm: StackVM::new(),
        }
    }
}
