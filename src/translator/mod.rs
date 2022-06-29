use std::path::Path;

use move_binary_format::{file_format::{FunctionHandleIndex, Bytecode}, CompiledModule};
use move_compiler::compiled_unit::NamedCompiledModule;
use move_ir_compiler::util::do_compile_module;
use anyhow::Error;

use crate::evm::ops::{Instruction, OpCode};
pub use code::CodeUnit;
pub use abi::MoveFunction;

mod code;
mod abi;
mod opcodes;

pub(crate) const ETH_MODULE_PATH: &'static str = "src/move/eth.mvir";

#[allow(dead_code)]
pub struct Translator {
    // module: NamedCompiledModule,
    module: CompiledModule,
}

impl Translator {
    pub fn new() -> Self {
        Translator {
            module: Translator::load_eth_opcodes_module(),
        }
    }

    fn load_eth_opcodes_module() -> CompiledModule {
        let (compiled_module, _) = do_compile_module(Path::new(ETH_MODULE_PATH), &[]);
        compiled_module
    }  

    pub fn translate(self, _bytecode: &str) -> Result<NamedCompiledModule, Error> 
    { 
        let eth_module = Self::load_eth_opcodes_module();
        dbg!(&eth_module);
        Err(Error::msg("test"))
    }

    #[allow(dead_code)]
    fn add_function(self, _module: &mut CompiledModule, _code: CodeUnit) -> Result<(), Error> {
        Ok(()) 
    }
}

pub trait Translate {
    fn translate(self) -> CodeUnit;
}

impl Translate for Instruction {
    fn translate(self) -> CodeUnit {
        match self.opcode() {
            OpCode::Stop => unimplemented!(),
            OpCode::Add => Bytecode::Add.into(),
            OpCode::Mul => Bytecode::Mul.into(),
            OpCode::Sub => Bytecode::Sub.into(),
            OpCode::Div => Bytecode::Div.into(),
            OpCode::SDiv => unimplemented!(),
            OpCode::Mod => Bytecode::Mod.into(),
            OpCode::SMod => unimplemented!(),
            OpCode::AddMod => vec![Bytecode::Add, Bytecode::Mod].into(),
            OpCode::MulMod => vec![Bytecode::Mul, Bytecode::Mod].into(),
            OpCode::Exp => unimplemented!(),
            OpCode::SignExtend => unimplemented!(),
            OpCode::Lt => unimplemented!(),
            OpCode::Gt => unimplemented!(),
            OpCode::SLt => unimplemented!(),
            OpCode::SGt => unimplemented!(),
            OpCode::EQ => Bytecode::Eq.into(),
            OpCode::IsZero => unimplemented!(),
            OpCode::And => Bytecode::And.into(),
            OpCode::Or => Bytecode::Or.into(),
            OpCode::Xor => Bytecode::Xor.into(),
            OpCode::Not => Bytecode::Not.into(),
            OpCode::Byte => unimplemented!(),
            OpCode::Shl => unimplemented!(),
            OpCode::Shr => unimplemented!(),
            OpCode::Sar => unimplemented!(),
            OpCode::Sha3 => unimplemented!(),
            OpCode::Addr => unimplemented!(),
            OpCode::Balance => unimplemented!(),
            OpCode::Origin => unimplemented!(),
            OpCode::Caller => unimplemented!(),
            OpCode::CallValue => unimplemented!(),
            OpCode::CallDataLoad => unimplemented!(),
            OpCode::CallDataSize => unimplemented!(),
            OpCode::CallDataCopy => unimplemented!(),
            OpCode::CodeSize => unimplemented!(),
            OpCode::CodeCopy => unimplemented!(),
            OpCode::GasPrice => unimplemented!(),
            OpCode::ExtCodeSize => unimplemented!(),
            OpCode::ExtCodeCopy => unimplemented!(),
            OpCode::ReturnDataSize => unimplemented!(),
            OpCode::ReturnDataCopy => unimplemented!(),
            OpCode::ExtCodeHash => unimplemented!(),
            OpCode::Blockhash => unimplemented!(),
            OpCode::Coinbase => unimplemented!(),
            OpCode::Timestamp => unimplemented!(),
            OpCode::Number => unimplemented!(),
            OpCode::Difficulty => unimplemented!(),
            OpCode::GasLimit => unimplemented!(),
            OpCode::Pop => Bytecode::Pop.into(),
            OpCode::MLoad => unimplemented!(),
            OpCode::MStore => unimplemented!(),
            OpCode::MStore8 => unimplemented!(),
            OpCode::SLoad => unimplemented!(),
            OpCode::SStore => unimplemented!(),
            OpCode::Jump => unimplemented!(),
            OpCode::JumpIf => unimplemented!(), // Bytecode::BrTrue
            OpCode::JumpDest => unimplemented!(),
            OpCode::PC => unimplemented!(),
            OpCode::MSize => unimplemented!(),
            OpCode::Gas => unimplemented!(),
            OpCode::Push(_bytes) => unimplemented!(),
            OpCode::Dup(_value) => unimplemented!(),
            OpCode::Swap(_pointer) => unimplemented!(),
            OpCode::Log(_value) => unimplemented!(),
            OpCode::Create => unimplemented!(),
            OpCode::Create2 => unimplemented!(),
            OpCode::Call => unimplemented!(),
            OpCode::CallCode => unimplemented!(),
            OpCode::StaticCall => {
                // Bytecode::Call(resolve_create_function_handle_index(*offset, &op, &ops)).into()
                Bytecode::Call(resolve_create_function_handle_index(&self)).into()
            }
            OpCode::DelegateCall => unimplemented!(),
            OpCode::Return => unimplemented!(),
            OpCode::Revert => unimplemented!(),
            OpCode::Invalid => unimplemented!(),
            OpCode::SelfDestruct => unimplemented!(),
        }
    }
}

/// Think it's should be runtime MVM related and based on order of module loading
fn resolve_create_function_handle_index(
    // offset: usize,
    op: &Instruction,
    // ops: &BTreeMap<usize, Instruction>,
) -> FunctionHandleIndex {
    match op.opcode() {
        OpCode::AddMod => Some(0),
        OpCode::MulMod => Some(1),
        _ => None,
    }
    .map(FunctionHandleIndex::new)
    .expect("Unimplemented EVM opcode")
}
