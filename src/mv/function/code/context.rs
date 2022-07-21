use crate::evm::bytecode::executor::execution::Var;
use crate::mv::function::code::stack::Stack;
use crate::mv::function::code::writer::{CodeWriter, FunctionCode};
use move_binary_format::file_format::{Bytecode, CodeOffset, LocalIndex, SignatureToken};
use std::collections::HashMap;
use anyhow::{anyhow, Error};

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

    pub fn overwrite(&mut self, offset: CodeOffset, code: Bytecode) {
        self.code.set_op(offset, code);
    }

    pub fn extend_code<I: IntoIterator<Item = Bytecode>>(&mut self, code: I) {
        self.code.extend(code);
    }

    pub fn push_stack(&mut self, tp: SignatureToken) -> SignatureToken {
        self.stack.push(tp.clone());
        tp
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
        self.local_mapping.insert(id, idx);
    }

    pub fn freeze(self) -> FunctionCode {
        self.code.freeze()
    }

    pub fn pc(&self) -> CodeOffset {
        self.code.pc()
    }

    pub fn borrow_local(&mut self, tp: SignatureToken) -> LocalIndex {
        self.code.borrow_local(tp)
    }

    pub fn release_local(&mut self, idx: LocalIndex) -> Option<SignatureToken> {
        self.code.release_local(idx)
    }

    pub fn set_var(&mut self, tp: SignatureToken) -> LocalIndex {
        self.code.set_var(tp)
    }

    pub fn move_local(&mut self, idx: LocalIndex) {
        self.code.move_local(idx)
    }

    pub fn st_var(&mut self, var: &Var) -> Result<(), Error> {
        let local = self
            .local_mapping
            .get(var)
            .ok_or_else(|| anyhow!("Unknown result variable:{}", var))?;
        self.code.move_local(*local);
        Ok(())
    }
}
