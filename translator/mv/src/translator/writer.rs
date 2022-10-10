use std::collections::HashMap;
use std::mem;

use anyhow::{bail, Error};
use eth::bytecode::hir::ir::Label;
use move_binary_format::file_format::{Bytecode, CodeOffset};

#[derive(Default, Debug)]
pub struct Code {
    code: Vec<Bytecode>,
    labels: HashMap<Label, CodeOffset>,
    jmps: HashMap<CodeOffset, Label>,
}

impl Code {
    /// Write a bytecode instruction to the code
    /// Use only for instructions that do not have a label!
    pub fn write(&mut self, bytecode: Bytecode) {
        assert!(bytecode.offset().is_none());
        self.code.push(bytecode);
    }

    pub fn label(&mut self, label: Label) {
        let offset = self.code.len() as CodeOffset;
        self.labels.insert(label, offset);
    }

    pub fn jmp(&mut self, label: Label, is_cnd: bool) {
        let offset = self.code.len() as CodeOffset;
        self.jmps.insert(offset, label);
        if is_cnd {
            self.code.push(Bytecode::BrTrue(offset));
        } else {
            self.code.push(Bytecode::Branch(offset));
        }
    }

    pub fn extend(&mut self, code: Vec<Bytecode>) {
        self.code.extend(code);
    }

    pub fn reset(&mut self) {
        self.code.clear();
        self.labels.clear();
        self.jmps.clear();
    }

    pub fn freeze(&mut self) -> Result<Vec<Bytecode>, Error> {
        let mut code = mem::take(&mut self.code);
        for (jmp, lbl) in self.jmps.iter() {
            if let Some(code) = code.get_mut(*jmp as usize) {
                *code = match code {
                    Bytecode::BrTrue(_) => Bytecode::BrTrue(self.labels[lbl]),
                    Bytecode::BrFalse(_) => Bytecode::BrFalse(self.labels[lbl]),
                    Bytecode::Branch(_) => Bytecode::Branch(self.labels[lbl]),
                    _ => bail!("unexpected bytecode {:?}", code),
                };
            } else {
                bail!("jmp to invalid code offset");
            }
        }
        self.reset();
        Ok(code)
    }

    pub fn get_opcode(&self, pc: CodeOffset) -> Option<&Bytecode> {
        self.code.get(pc as usize)
    }
}
