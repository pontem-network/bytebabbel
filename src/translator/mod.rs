use std::{io::{Read, Write}, path::Path};

use move_binary_format::{file_format::{FunctionHandleIndex, Bytecode}, CompiledModule};
use move_ir_compiler::util::do_compile_module;
use crate::{Instruction, error};
pub use code::CodeUnit;

mod code;

pub struct Translator;

impl Translator {
    pub fn new() -> Self {
        Translator
    }

    fn load_eth_opcodes_module(self) -> CompiledModule {
        let (compiled_module, _) = do_compile_module(Path::new("./src/move/eth.mvir"), &[]);
        compiled_module
    }  

    pub fn translate<R, W>(self, mut reader: R, mut writer: W) -> Result<(), error::Error> 
    where
        R: Read,
        W: Write,
    { 
        let eth_module = self.load_eth_opcodes_module();
        dbg!(&eth_module);
        Ok(())
    }
}

pub trait Translate {
    fn translate(self) -> CodeUnit;
}

impl Translate for Instruction {
    fn translate(self) -> CodeUnit {
        match self {
            Instruction::Stop => unimplemented!(),
            Instruction::Add => Bytecode::Add.into(),
            Instruction::Mul => Bytecode::Mul.into(),
            Instruction::Sub => Bytecode::Sub.into(),
            Instruction::Div => Bytecode::Div.into(),
            Instruction::SDiv => unimplemented!(),
            Instruction::Mod => Bytecode::Mod.into(),
            Instruction::SMod => unimplemented!(),
            Instruction::AddMod => vec![Bytecode::Add, Bytecode::Mod].into(),
            Instruction::MulMod => vec![Bytecode::Mul, Bytecode::Mod].into(),
            Instruction::Exp => unimplemented!(),
            Instruction::SignExtend => unimplemented!(),
            Instruction::Lt => unimplemented!(),
            Instruction::Gt => unimplemented!(),
            Instruction::SLt => unimplemented!(),
            Instruction::SGt => unimplemented!(),
            Instruction::EQ => Bytecode::Eq.into(),
            Instruction::IsZero => unimplemented!(),
            Instruction::And => Bytecode::And.into(),
            Instruction::Or => Bytecode::Or.into(),
            Instruction::Xor => Bytecode::Xor.into(),
            Instruction::Not => Bytecode::Not.into(),
            Instruction::Byte => unimplemented!(),
            Instruction::Shl => unimplemented!(),
            Instruction::Shr => unimplemented!(),
            Instruction::Sar => unimplemented!(),
            Instruction::Sha3 => unimplemented!(),
            Instruction::Addr => unimplemented!(),
            Instruction::Balance => unimplemented!(),
            Instruction::Origin => unimplemented!(),
            Instruction::Caller => unimplemented!(),
            Instruction::CallValue => unimplemented!(),
            Instruction::CallDataLoad => unimplemented!(),
            Instruction::CallDataSize => unimplemented!(),
            Instruction::CallDataCopy => unimplemented!(),
            Instruction::CodeSize => unimplemented!(),
            Instruction::CodeCopy => unimplemented!(),
            Instruction::GasPrice => unimplemented!(),
            Instruction::ExtCodeSize => unimplemented!(),
            Instruction::ExtCodeCopy => unimplemented!(),
            Instruction::ReturnDataSize => unimplemented!(),
            Instruction::ReturnDataCopy => unimplemented!(),
            Instruction::ExtCodeHash => unimplemented!(),
            Instruction::Blockhash => unimplemented!(),
            Instruction::Coinbase => unimplemented!(),
            Instruction::Timestamp => unimplemented!(),
            Instruction::Number => unimplemented!(),
            Instruction::Difficulty => unimplemented!(),
            Instruction::GasLimit => unimplemented!(),
            Instruction::Pop => Bytecode::Pop.into(),
            Instruction::MLoad => unimplemented!(),
            Instruction::MStore => unimplemented!(),
            Instruction::MStore8 => unimplemented!(),
            Instruction::SLoad => unimplemented!(),
            Instruction::SStore => unimplemented!(),
            Instruction::Jump => unimplemented!(),
            Instruction::JumpIf => unimplemented!(), // Bytecode::BrTrue
            Instruction::JumpDest => unimplemented!(),
            Instruction::PC => unimplemented!(),
            Instruction::MSize => unimplemented!(),
            Instruction::Gas => unimplemented!(),
            Instruction::Push(bytes) => unimplemented!(),
            Instruction::Dup(value) => unimplemented!(),
            Instruction::Swap(pointer) => unimplemented!(),
            Instruction::Log(value) => unimplemented!(),
            Instruction::Create => unimplemented!(),
            Instruction::Create2 => unimplemented!(),
            Instruction::Call => unimplemented!(),
            Instruction::CallCode => unimplemented!(),
            Instruction::StaticCall => {
                // Bytecode::Call(resolve_create_function_handle_index(*offset, &op, &ops)).into()
                Bytecode::Call(resolve_create_function_handle_index(&self)).into()
            }
            Instruction::DelegateCall => unimplemented!(),
            Instruction::Return => unimplemented!(),
            Instruction::Revert => unimplemented!(),
            Instruction::Invalid => unimplemented!(),
            Instruction::SelfDestruct => unimplemented!(),
        }
    }
}

/// Think it's should be runtime MVM related and based on order of module loading
fn resolve_create_function_handle_index(
    // offset: usize,
    op: &Instruction,
    // ops: &BTreeMap<usize, Instruction>,
) -> FunctionHandleIndex {
    match op {
        Instruction::AddMod => Some(0),
        Instruction::MulMod => Some(1),
        _ => None,
    }
    .map(FunctionHandleIndex::new)
    .expect("Unimplemented EVM opcode")
}
