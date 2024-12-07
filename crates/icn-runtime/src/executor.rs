use crate::vm::VM;

pub struct Executor {
    vm: VM,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            vm: VM::new()
        }
    }
}
