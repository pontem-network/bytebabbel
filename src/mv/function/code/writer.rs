use move_binary_format::file_format::{
    Bytecode, CodeOffset, LocalIndex, SignatureToken, StructFieldInformation,
};
use std::collections::{HashMap, VecDeque};

pub struct CodeWriter {
    code: Vec<Bytecode>,
    locals: HashMap<SignatureToken, Locals>,
    local_seq: LocalIndex,
    params_count: LocalIndex,
    trace: bool,
}

impl CodeWriter {
    pub fn new(params_count: usize, trace: bool) -> CodeWriter {
        CodeWriter {
            code: vec![],
            locals: Default::default(),
            local_seq: params_count as LocalIndex,
            params_count: params_count as LocalIndex,
            trace,
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

    pub fn set_var(&mut self, tp: SignatureToken) -> LocalIndex {
        let idx = self.borrow_local(tp);
        self.stack.pop();
        self.push(Bytecode::StLoc(idx));
        idx
    }

    pub fn move_local(&mut self, idx: LocalIndex) {
        if let Some(tp) = self.release_local(idx) {
            self.stack.push(tp);
            self.push(Bytecode::MoveLoc(idx));
        }
    }

    pub fn push(&mut self, code: Bytecode) {
        self.code.push(code);
    }

    pub fn set_op(&mut self, idx: CodeOffset, op_code: Bytecode) {
        self.code[idx as usize] = op_code;
    }

    pub fn extend<I: IntoIterator<Item = Bytecode>>(&mut self, code: I) {
        for bytecode in code {
            self.push(bytecode);
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

        if self.trace {
            for (local, tkn) in locals.iter().enumerate() {
                println!("loc_{:?} = {:?}", local, tkn);
            }

            for (idx, code) in self.code.iter().enumerate() {
                println!("{idx}: {code:?}");
            }
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
            return true;
        } else {
            return false;
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
