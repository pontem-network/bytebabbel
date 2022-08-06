use crate::bytecode::hir::ir::var::VarId;
use crate::bytecode::hir::mem::Memory;
use crate::bytecode::hir::stack::Stack;
use crate::bytecode::types::{Env, Function, U256};
use crate::BlockId;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Context {
    address: U256,
    mem: Memory,
    stack: Stack,
    env: Rc<Env>,
    loop_input: HashMap<BlockId, Stack>,
}

impl Context {
    pub fn new(fun: Function, contract_address: U256) -> Context {
        Context {
            address: contract_address,
            mem: Memory::default(),
            stack: Stack::default(),
            env: Rc::new(Env::new(fun)),
            loop_input: Default::default(),
        }
    }

    pub fn pop_stack(&mut self, pops: usize) -> Vec<VarId> {
        self.stack.pop(pops)
    }

    pub fn push_stack(&mut self, to_push: Vec<VarId>) {
        self.stack.push(to_push)
    }

    pub fn env(&self) -> &Env {
        self.env.as_ref()
    }

    pub fn address(&self) -> U256 {
        self.address
    }

    pub fn mem_load(&mut self, offset: U256) -> Option<VarId> {
        self.mem.static_load(offset)
    }

    pub fn mem_store(&mut self, offset: U256, val: VarId) {
        self.mem.static_store(offset, val)
    }

    pub fn create_loop(&mut self, block_id: BlockId) {
        self.loop_input.insert(block_id, self.stack.clone());
    }

    pub fn get_loop(&self, block_id: &BlockId) -> Option<&Stack> {
        self.loop_input.get(block_id)
    }

    pub fn map_stack(&self, origin: &Stack) -> Vec<MapStackItem> {
        let mut mapping = Vec::with_capacity(origin.stack.len());

        let mut last_origin_index = origin.stack.len() - 1;
        let mut last_new_index = self.stack.stack.len() - 1;
        loop {
            let new = self.stack.stack.get(last_new_index);
            let origin = origin.stack.get(last_origin_index);

            if let (Some(new), Some(origin)) = (new, origin) {
                if new != origin {
                    mapping.push(MapStackItem {
                        origin: *origin,
                        new: *new,
                    });
                }
            } else {
                break;
            }
            if last_origin_index == 0 || last_new_index == 0 {
                break;
            }
            last_origin_index -= 1;
            last_new_index -= 1;
        }

        mapping
    }
}

pub struct MapStackItem {
    pub origin: VarId,
    pub new: VarId,
}
