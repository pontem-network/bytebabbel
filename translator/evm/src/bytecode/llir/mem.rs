use crate::bytecode::llir::ir::var::VarId;
use crate::bytecode::types::U256;
use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
pub struct Memory {
    aligned_mem: HashMap<U256, VarId>,
}

impl Memory {
    pub fn store_aligned(&mut self, offset: U256, val: VarId) {
        assert_eq!(offset % 32, U256::zero());
        self.aligned_mem.insert(offset, val);
    }

    pub fn load_aligned(&mut self, offset: U256) -> Option<VarId> {
        assert_eq!(offset % 32, U256::zero());
        self.aligned_mem.get(&offset).cloned()
    }
}
