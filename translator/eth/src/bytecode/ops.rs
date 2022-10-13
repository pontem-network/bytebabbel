use std::fmt::Debug;

use crate::bytecode::instruction::Instruction;

pub struct InstructionIter {
    offset: usize,
    buffer: Vec<u8>,
}

impl InstructionIter {
    pub fn new(buffer: Vec<u8>) -> InstructionIter {
        Self { offset: 0, buffer }
    }
}

impl Iterator for InstructionIter {
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.len() <= self.offset {
            None
        } else {
            let op_code = OpCode::from(&self.buffer[self.offset..]);
            let offset = self.offset;
            self.offset += op_code.size();
            Some(Instruction::new(offset, op_code))
        }
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum OpCode {
    Stop,
    Add,
    Mul,
    Sub,
    Div,
    SDiv,
    Mod,
    SMod,
    AddMod,
    MulMod,
    Exp,
    SignExtend,
    Lt,
    Gt,
    SLt,
    SGt,
    EQ,
    IsZero,
    And,
    Or,
    Xor,
    Not,
    Byte,
    Shl,
    Shr,
    // logical shift right
    Sar,
    // arithmetic shift right
    Sha3,
    Addr,
    Balance,
    Origin,
    Caller,
    CallValue,
    CallDataLoad,
    CallDataSize,
    CallDataCopy,
    CodeSize,
    CodeCopy,
    GasPrice,
    ExtCodeSize,
    ExtCodeCopy,
    ReturnDataSize,
    ReturnDataCopy,
    ExtCodeHash,
    Blockhash,
    Coinbase,
    Timestamp,
    Number,
    Difficulty,
    GasLimit,
    Pop,
    MLoad,
    MStore,
    MStore8,
    SLoad,
    SStore,
    Jump,
    JumpIf,
    PC,
    MSize,
    Gas,
    JumpDest,
    Push(Vec<u8>),
    Dup(usize),
    Swap(usize),
    Log(usize),
    Create,
    Call,
    CallCode,
    Return,
    DelegateCall,
    Create2,
    Revert,
    StaticCall,
    Invalid(u8),
    SelfDestruct,
}

impl OpCode {
    pub fn size(&self) -> usize {
        match self {
            Self::Push(ref a) => a.len() + 1,
            _ => 1,
        }
    }

    pub fn writes_to_memory(&self) -> bool {
        matches!(
            self,
            Self::MStore
                | Self::MStore8
                | Self::CallDataCopy
                | Self::CodeCopy
                | Self::ExtCodeCopy
                | Self::ReturnDataCopy
                | Self::Call
                | Self::StaticCall
                | Self::DelegateCall
                | Self::CallCode
        )
    }

    pub fn reads_from_memory(&self) -> bool {
        matches!(
            self,
            Self::MLoad
                | Self::Create
                | Self::Call
                | Self::StaticCall
                | Self::DelegateCall
                | Self::CallCode
                | Self::Return
                | Self::Revert
        )
    }

    pub fn writes_to_storage(&self) -> bool {
        matches!(self, Self::SStore)
    }

    pub fn reads_from_storage(&self) -> bool {
        matches!(self, Self::SLoad)
    }

    pub fn halts_execution(&self) -> bool {
        matches!(
            self,
            Self::Return | Self::Stop | Self::Invalid(_) | Self::SelfDestruct | Self::Revert
        )
    }

    pub fn is_jump(&self) -> bool {
        matches!(self, Self::Jump | Self::JumpIf)
    }

    pub fn as_push(&self) -> Option<&[u8]> {
        match self {
            OpCode::Push(push) => Some(push.as_slice()),
            _ => None,
        }
    }

    pub fn ends_basic_block(&self) -> bool {
        matches!(
            self,
            Self::Return
                | Self::Stop
                | Self::Invalid(_)
                | Self::SelfDestruct
                | Self::Revert
                | Self::Jump
                | Self::JumpIf
        )
    }

    pub fn pops(&self) -> usize {
        match self {
            Self::Stop
            | Self::Addr
            | Self::Origin
            | Self::Caller
            | Self::CallValue
            | Self::CallDataSize
            | Self::CodeSize
            | Self::GasPrice
            | Self::Coinbase
            | Self::Timestamp
            | Self::Number
            | Self::Difficulty
            | Self::GasLimit
            | Self::PC
            | Self::MSize
            | Self::Gas
            | Self::JumpDest
            | Self::Push(..)
            | Self::Invalid(_)
            | Self::ReturnDataSize => 0,
            Self::IsZero
            | Self::Not
            | Self::Balance
            | Self::CallDataLoad
            | Self::ExtCodeSize
            | Self::Blockhash
            | Self::Pop
            | Self::MLoad
            | Self::SLoad
            | Self::Jump
            | Self::SelfDestruct
            | Self::ExtCodeHash => 1,
            Self::Add
            | Self::Mul
            | Self::Sub
            | Self::Div
            | Self::SDiv
            | Self::Mod
            | Self::SMod
            | Self::Exp
            | Self::SignExtend
            | Self::Lt
            | Self::Gt
            | Self::SLt
            | Self::SGt
            | Self::EQ
            | Self::And
            | Self::Or
            | Self::Xor
            | Self::Byte
            | Self::Sha3
            | Self::MStore
            | Self::MStore8
            | Self::SStore
            | Self::JumpIf
            | Self::Return
            | Self::Revert
            | Self::Shl
            | Self::Shr
            | Self::Sar => 2,
            Self::AddMod
            | Self::MulMod
            | Self::CallDataCopy
            | Self::CodeCopy
            | Self::Create
            | Self::ReturnDataCopy
            | Self::Create2 => 3,
            Self::ExtCodeCopy => 4,
            Self::DelegateCall | Self::StaticCall => 6,
            Self::Call | Self::CallCode => 7,
            Self::Dup(u) => *u,
            Self::Swap(u) => *u + 1,
            Self::Log(u) => u + 2,
        }
    }

    pub fn pushes(&self) -> usize {
        match self {
            Self::Stop
            | Self::CallDataCopy
            | Self::CodeCopy
            | Self::ExtCodeCopy
            | Self::Pop
            | Self::MStore
            | Self::MStore8
            | Self::SStore
            | Self::Jump
            | Self::JumpIf
            | Self::JumpDest
            | Self::Log(..)
            | Self::Return
            | Self::Invalid(_)
            | Self::SelfDestruct
            | Self::ReturnDataCopy
            | Self::Revert => 0,
            Self::Dup(u) => u + 1,
            Self::Swap(u) => *u + 1,
            _ => 1,
        }
    }
}

impl From<&[u8]> for OpCode {
    fn from(buff: &[u8]) -> Self {
        let opcode = buff[0];
        match opcode {
            0x00 => OpCode::Stop,
            0x01 => OpCode::Add,
            0x02 => OpCode::Mul,
            0x03 => OpCode::Sub,
            0x04 => OpCode::Div,
            0x05 => OpCode::SDiv,
            0x06 => OpCode::Mod,
            0x07 => OpCode::SMod,
            0x08 => OpCode::AddMod,
            0x09 => OpCode::MulMod,
            0x0a => OpCode::Exp,
            0x0b => OpCode::SignExtend,
            0x10 => OpCode::Lt,
            0x11 => OpCode::Gt,
            0x12 => OpCode::SLt,
            0x13 => OpCode::SGt,
            0x14 => OpCode::EQ,
            0x15 => OpCode::IsZero,
            0x16 => OpCode::And,
            0x17 => OpCode::Or,
            0x18 => OpCode::Xor,
            0x19 => OpCode::Not,
            0x1a => OpCode::Byte,
            0x1b => OpCode::Shl,
            0x1c => OpCode::Shr,
            0x1d => OpCode::Sar,
            0x20 => OpCode::Sha3,
            0x30 => OpCode::Addr,
            0x31 => OpCode::Balance,
            0x32 => OpCode::Origin,
            0x33 => OpCode::Caller,
            0x34 => OpCode::CallValue,
            0x35 => OpCode::CallDataLoad,
            0x36 => OpCode::CallDataSize,
            0x37 => OpCode::CallDataCopy,
            0x38 => OpCode::CodeSize,
            0x39 => OpCode::CodeCopy,
            0x3a => OpCode::GasPrice,
            0x3b => OpCode::ExtCodeSize,
            0x3c => OpCode::ExtCodeCopy,
            0x3f => OpCode::ExtCodeHash,
            0x3d => OpCode::ReturnDataSize,
            0x3e => OpCode::ReturnDataCopy,
            0x40 => OpCode::Blockhash,
            0x41 => OpCode::Coinbase,
            0x42 => OpCode::Timestamp,
            0x43 => OpCode::Number,
            0x44 => OpCode::Difficulty,
            0x45 => OpCode::GasLimit,
            0x50 => OpCode::Pop,
            0x51 => OpCode::MLoad,
            0x52 => OpCode::MStore,
            0x53 => OpCode::MStore8,
            0x54 => OpCode::SLoad,
            0x55 => OpCode::SStore,
            0x56 => OpCode::Jump,
            0x57 => OpCode::JumpIf,
            0x58 => OpCode::PC,
            0x59 => OpCode::MSize,
            0x5a => OpCode::Gas,
            0x5b => OpCode::JumpDest,
            0x60 | 0x61 | 0x62 | 0x63 | 0x64 | 0x65 | 0x66 | 0x67 | 0x68 | 0x69 | 0x6a | 0x6b
            | 0x6c | 0x6d | 0x6e | 0x6f => {
                OpCode::Push(read_n_bytes(&buff[1..], 1 + (opcode & 0x0f) as usize))
            }
            0x70 | 0x71 | 0x72 | 0x73 | 0x74 | 0x75 | 0x76 | 0x77 | 0x78 | 0x79 | 0x7a | 0x7b
            | 0x7c | 0x7d | 0x7e | 0x7f => {
                OpCode::Push(read_n_bytes(&buff[1..], (0x11 + (opcode & 0x0f)) as usize))
            }
            0x80 => OpCode::Dup(1),
            0x81 => OpCode::Dup(2),
            0x82 => OpCode::Dup(3),
            0x83 => OpCode::Dup(4),
            0x84 => OpCode::Dup(5),
            0x85 => OpCode::Dup(6),
            0x86 => OpCode::Dup(7),
            0x87 => OpCode::Dup(8),
            0x88 => OpCode::Dup(9),
            0x89 => OpCode::Dup(10),
            0x8a => OpCode::Dup(11),
            0x8b => OpCode::Dup(12),
            0x8c => OpCode::Dup(13),
            0x8d => OpCode::Dup(14),
            0x8e => OpCode::Dup(15),
            0x8f => OpCode::Dup(16),
            0x90 => OpCode::Swap(1),
            0x91 => OpCode::Swap(2),
            0x92 => OpCode::Swap(3),
            0x93 => OpCode::Swap(4),
            0x94 => OpCode::Swap(5),
            0x95 => OpCode::Swap(6),
            0x96 => OpCode::Swap(7),
            0x97 => OpCode::Swap(8),
            0x98 => OpCode::Swap(9),
            0x99 => OpCode::Swap(10),
            0x9a => OpCode::Swap(11),
            0x9b => OpCode::Swap(12),
            0x9c => OpCode::Swap(13),
            0x9d => OpCode::Swap(14),
            0x9e => OpCode::Swap(15),
            0x9f => OpCode::Swap(16),
            0xa0 => OpCode::Log(0),
            0xa1 => OpCode::Log(1),
            0xa2 => OpCode::Log(2),
            0xa3 => OpCode::Log(3),
            0xa4 => OpCode::Log(4),
            0xf0 => OpCode::Create,
            0xf1 => OpCode::Call,
            0xf2 => OpCode::CallCode,
            0xf3 => OpCode::Return,
            0xf4 => OpCode::DelegateCall,
            0xfb => OpCode::Create2,
            0xfd => OpCode::Revert,
            0xfa => OpCode::StaticCall,
            0xff => OpCode::SelfDestruct,
            _ => OpCode::Invalid(opcode),
        }
    }
}

fn read_n_bytes(buffer: &[u8], n: usize) -> Vec<u8> {
    if buffer.len() < n {
        buffer.to_vec()
    } else {
        buffer[..n].to_vec()
    }
}
