use anyhow::{bail, Error};
use evm::bytecode::block::BlockId;
use move_binary_format::file_format::{Bytecode, CodeOffset};
use std::collections::HashMap;
use std::mem;

#[derive(Default)]
pub struct Writer {
    code: Vec<Bytecode>,
    labels: HashMap<BlockId, CodeOffset>,
    jmp_label: HashMap<CodeOffset, BlockId>,
    jmps: Vec<CodeOffset>,
}

impl Writer {
    pub fn write(&mut self, bytecode: Bytecode) {
        self.code.push(bytecode);
    }

    pub fn extend(&mut self, mut other: Writer) -> Result<(), Error> {
        let prefix = self.pc();

        self.labels.extend(
            other
                .labels
                .into_iter()
                .map(|(id, offset)| (id, offset + prefix)),
        );

        self.jmp_label.extend(
            other
                .jmp_label
                .into_iter()
                .map(|(offset, id)| (offset + prefix, id)),
        );

        for jmp in other.jmps.iter() {
            self.jmps.push(*jmp + prefix);
            if let Some(code) = other.code.get_mut(*jmp as usize) {
                *code = match code {
                    Bytecode::BrTrue(pc) => Bytecode::BrTrue(*pc + prefix),
                    Bytecode::BrFalse(pc) => Bytecode::BrFalse(*pc + prefix),
                    Bytecode::Branch(pc) => Bytecode::Branch(*pc + prefix),
                    _ => bail!("unexpected bytecode {:?}", code),
                };
            } else {
                bail!("jmp to invalid code offset");
            }
        }

        self.code.extend(other.code);
        Ok(())
    }

    pub fn reset(&mut self) {
        self.code.clear();
        self.labels.clear();
        self.jmp_label.clear();
        self.jmps.clear();
    }

    pub fn freeze(&mut self) -> Result<Vec<Bytecode>, Error> {
        for (jmp_loc, label) in &self.jmp_label {
            let label_offset = self.labels.get(label).ok_or_else(|| {
                anyhow::anyhow!(
                    "jmp to invalid code offset {:?} (label {:?})",
                    jmp_loc,
                    label
                )
            })?;
            let code = self
                .code
                .get_mut(*jmp_loc as usize)
                .ok_or_else(|| anyhow::anyhow!("jmp to invalid code offset {:?}", jmp_loc))?;
            *code = match code {
                Bytecode::Branch(_) => Bytecode::Branch(*label_offset),
                _ => bail!("unexpected bytecode {:?}", code),
            }
        }

        Ok(mem::take(&mut self.code))
    }

    pub fn swap(&mut self, mut other: Writer) -> Writer {
        mem::swap(self, &mut other);
        other
    }

    pub fn pc(&self) -> CodeOffset {
        self.code.len() as CodeOffset
    }

    pub fn create_label(&mut self, label: BlockId) {
        self.labels.insert(label, self.pc());
    }

    pub fn label_offset(&self, label: BlockId) -> Option<CodeOffset> {
        self.labels.get(&label).cloned()
    }

    pub fn mark_jmp_to_label(&mut self, label: BlockId) {
        self.jmp_label.insert(self.pc(), label);
        self.code.push(Bytecode::Branch(0));
    }

    pub fn mark_jmp(&mut self) {
        self.jmps.push(self.pc());
    }
}
