use crate::bytecode::hir::ir::var::VarId;
use crate::bytecode::types::U256;
use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
pub struct Memory {
    aligned_mem: HashMap<U256, VarId>,
}

impl Memory {
    pub fn static_store(&mut self, offset: U256, val: VarId) {
        self.aligned_mem.insert(offset, val);
    }

    pub fn static_load(&mut self, offset: U256) -> Option<VarId> {
        self.aligned_mem.get(&offset).cloned()
    }
}
