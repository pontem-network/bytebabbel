use crate::translator::writer::Writer;
use evm::bytecode::mir::ir::math::Operation;
use evm::bytecode::mir::ir::types::Value;
use move_binary_format::file_format::{Bytecode, LocalIndex};

impl Writer {
    pub fn push_val(&mut self, value: &Value) {
        let op_code = match value {
            Value::U128(val) => Bytecode::LdU128(*val),
            Value::Bool(val) => {
                if *val {
                    Bytecode::LdTrue
                } else {
                    Bytecode::LdFalse
                }
            }
        };
        self.write(op_code);
    }

    pub fn ld_var(&mut self, idx: LocalIndex) {
        self.write(Bytecode::CopyLoc(idx));
    }

    pub fn op(&mut self, cmd: Operation) {
        if cmd == Operation::Shl || cmd == Operation::Shr {
            self.write(Bytecode::CastU8);
        }

        let code = match cmd {
            Operation::Add => Bytecode::Add,
            Operation::Sub => Bytecode::Sub,
            Operation::Mul => Bytecode::Mul,
            Operation::Mod => Bytecode::Mod,
            Operation::Div => Bytecode::Div,
            Operation::BitOr => Bytecode::BitOr,
            Operation::BitAnd => Bytecode::BitAnd,
            Operation::Xor => Bytecode::Xor,
            Operation::Or => Bytecode::Or,
            Operation::And => Bytecode::And,
            Operation::Not => Bytecode::Not,
            Operation::Eq => Bytecode::Eq,
            Operation::Neq => Bytecode::Neq,
            Operation::Lt => Bytecode::Lt,
            Operation::Gt => Bytecode::Gt,
            Operation::Le => Bytecode::Le,
            Operation::Ge => Bytecode::Ge,
            Operation::Shl => Bytecode::Shl,
            Operation::Shr => Bytecode::Shr,
        };
        self.write(code);
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
