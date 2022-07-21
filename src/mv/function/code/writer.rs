use move_binary_format::file_format::{Bytecode, CodeOffset, LocalIndex, SignatureToken};
use std::collections::HashMap;

pub struct CodeWriter {
    code: Vec<Bytecode>,
    locals: HashMap<SignatureToken, Locals>,
    local_seq: LocalIndex,
    params_count: LocalIndex,
}

impl CodeWriter {
    pub fn new(params_count: usize) -> CodeWriter {
        CodeWriter {
            code: vec![],
            locals: Default::default(),
            local_seq: params_count as LocalIndex,
            params_count: params_count as LocalIndex,
        }
    }

    pub fn borrow_local(&mut self, tp: SignatureToken) -> LocalIndex {
        let locals = self.locals.entry(tp).or_default();
        if let Some(idx) = locals.borrow() {
            idx
        } else {
            let idx = self.local_seq;
            self.local_seq += 1;
            locals.new_borrowed(idx);
            idx
        }
    }

    pub fn release_local(&mut self, idx: LocalIndex) -> Option<SignatureToken> {
        for (tp, locals) in self.locals.iter_mut() {
            if locals.release(idx) {
                return Some(tp.clone());
            }
        }
        None
    }

    pub fn local_type(&self, idx: LocalIndex) -> Option<SignatureToken> {
        for (tp, locals) in self.locals.iter() {
            if locals.contains(idx) {
                return Some(tp.clone());
            }
        }
        None
    }

    pub fn push(&mut self, code: Bytecode) {
        self.code.push(code);
    }

    pub fn update_jmp_pc(&mut self, idx: CodeOffset, new_pc: CodeOffset) {
        let code = &mut self.code[idx as usize];
        match code {
            Bytecode::Branch(new_offset)
            | Bytecode::BrFalse(new_offset)
            | Bytecode::BrTrue(new_offset) => {
                *new_offset = new_pc;
            }
            _ => panic!("Expected Jmp"),
        }
    }

    pub fn pc(&self) -> CodeOffset {
        self.code.len() as CodeOffset
    }

    pub fn freeze(self) -> FunctionCode {
        let locals = (self.params_count..self.local_seq)
            .map(|id| {
                self.locals
                    .iter()
                    .find(|(_, locals)| locals.contains(id))
                    .map(|(tkn, _)| tkn.clone())
                    .unwrap()
            })
            .collect::<Vec<_>>();

        for (local, tkn) in locals.iter().enumerate() {
            log::trace!("loc_{:?} = {:?}", local + self.params_count as usize, tkn);
        }

        for (idx, code) in self.code.iter().enumerate() {
            log::trace!("{idx}: {code:?}");
        }

        FunctionCode {
            code: self.code,
            locals,
        }
    }
}

#[derive(Default)]
pub struct Locals {
    free: Vec<LocalIndex>,
    borrowed: Vec<LocalIndex>,
}

impl Locals {
    pub fn borrow(&mut self) -> Option<LocalIndex> {
        if let Some(free) = self.free.pop() {
            self.borrowed.push(free);
            Some(free)
        } else {
            None
        }
    }

    pub fn new_borrowed(&mut self, id: LocalIndex) {
        self.borrowed.push(id);
    }

    pub fn release(&mut self, id: LocalIndex) -> bool {
        let borrowed_idx = self
            .borrowed
            .iter()
            .enumerate()
            .find(|(_, b)| **b == id)
            .map(|(i, _)| i);

        if let Some(borrowed_idx) = borrowed_idx {
            self.borrowed.remove(borrowed_idx);
            self.free.push(id);
            true
        } else {
            false
        }
    }

    pub fn contains(&self, idx: LocalIndex) -> bool {
        self.free.contains(&idx) || self.borrowed.contains(&idx)
    }
}

pub struct FunctionCode {
    pub code: Vec<Bytecode>,
    pub locals: Vec<SignatureToken>,
}
