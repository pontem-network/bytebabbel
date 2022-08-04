use crate::bytecode::executor::env::Env;
use crate::bytecode::executor::mem::Memory;
use crate::bytecode::executor::stack::Stack;
use crate::Function;
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
}
