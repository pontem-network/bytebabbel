use crate::translator::writer::Code;
use move_binary_format::file_format::{Bytecode, LocalIndex};

impl Code {
    pub fn ld_var(&mut self, idx: LocalIndex) {
        self.write(Bytecode::CopyLoc(idx));
    }

    pub fn set_var(&mut self, idx: LocalIndex) {
        self.write(Bytecode::StLoc(idx));
    }

    pub fn abort(&mut self, code: u8) {
        self.write(Bytecode::LdU64(code as u64));
        self.write(Bytecode::Abort);
    }

    pub fn is_final(&self) -> bool {
        self.get_opcode(self.pc() - 1)
            .map(|op_code| matches!(op_code, Bytecode::Ret | Bytecode::Abort))
            .unwrap_or_default()
    }
}
