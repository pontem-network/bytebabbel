use move_binary_format::file_format::{Bytecode, LocalIndex};

use crate::translator::writer::Code;

impl Code {
    pub fn copy_loc(&mut self, idx: LocalIndex) {
        self.write(Bytecode::CopyLoc(idx));
    }

    pub fn move_loc(&mut self, idx: LocalIndex) {
        self.write(Bytecode::MoveLoc(idx));
    }

    pub fn assign(&mut self, idx: LocalIndex) {
        self.write(Bytecode::StLoc(idx));
    }

    pub fn abort(&mut self, code: u8) {
        self.write(Bytecode::LdU64(code as u64));
        self.write(Bytecode::Abort);
    }
}
