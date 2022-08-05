use crate::bytecode::llir::mem::Memory;
use crate::bytecode::llir::stack::{Stack, StackFrame};
use crate::bytecode::types::{Env, Function};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Context {
    mem: Memory,
    stack: Stack,
    env: Rc<Env>,
}

impl Context {
    pub fn new(fun: Function) -> Context {
        Context {
            mem: Memory::default(),
            stack: Stack::default(),
            env: Rc::new(Env::new(fun)),
        }
    }

    pub fn pop_stack(&mut self, pops: usize) -> Vec<StackFrame> {
        self.stack.pop(pops)
    }

    pub fn env(&self) -> &Env {
        self.env.as_ref()
    }
}
