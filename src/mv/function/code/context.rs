use crate::evm::bytecode::executor::execution::Var;
use crate::mv::function::code::intrinsic::math::MathModel;
use crate::mv::function::code::stack::Stack;
use crate::mv::function::code::writer::CodeWriter;
use move_binary_format::file_format::{Bytecode, LocalIndex, SignatureToken};
use std::collections::HashMap;
use std::ops::Deref;

pub struct Context {
    pub code: CodeWriter,
    pub stack: Stack,
    pub local_mapping: HashMap<Var, LocalIndex>,
}

impl Context {
    pub fn new(code: CodeWriter) -> Context {
        Context {
            code,
            stack: Default::default(),
            local_mapping: Default::default(),
        }
    }

    pub fn write_code(&mut self, code: Bytecode) {
        self.code.push(code);
    }

    pub fn extend_code<I: IntoIterator<Item = Bytecode>>(&mut self, code: I) {
        self.code.extend(code);
    }

    pub fn push_stack(&mut self, tp: SignatureToken) {
        self.stack.push(tp);
    }

    pub fn pop_stack(&mut self) -> SignatureToken {
        self.stack.pop()
    }

    pub fn pop2_stack(&mut self) -> [SignatureToken; 2] {
        self.stack.pop2()
    }

    pub fn set_global_var(&mut self, id: Var, tp: SignatureToken) {
        self.stack.pop();
        let idx = self.code.set_var(tp);
        self.local_mapping.insert(*id, idx);
    }
}
