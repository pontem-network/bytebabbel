use crate::bytecode::llir::ir::var::VarId;
use crate::bytecode::llir::mem::Memory;
use crate::bytecode::llir::stack::Stack;
use crate::bytecode::types::{Env, Function, U256};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Context {
    address: U256,
    mem: Memory,
    stack: Stack,
    env: Rc<Env>,
}

impl Context {
    pub fn new(fun: Function, contract_address: U256) -> Context {
        Context {
            address: contract_address,
            mem: Memory::default(),
            stack: Stack::default(),
            env: Rc::new(Env::new(fun)),
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
        self.mem.load_aligned(offset)
    }

    pub fn mem_store(&mut self, offset: U256, val: VarId) {
        self.mem.store_aligned(offset, val)
    }
}
