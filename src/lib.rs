use std::collections::BTreeMap;
use std::io::Read;
use std::io::Write;

use move_binary_format::CompiledModule;
use move_binary_format::file_format::AddressIdentifierIndex;
use move_binary_format::file_format::FunctionHandleIndex;
use move_binary_format::file_format::IdentifierIndex;
use move_binary_format::file_format::ModuleHandle;
use move_binary_format::file_format::ModuleHandleIndex;
use move_binary_format::file_format_common::VERSION_MAX;
use move_binary_format::{
    binary_views::BinaryIndexedView,
    control_flow_graph::{ControlFlowGraph, VMControlFlowGraph},
    file_format::{
        Ability, AbilitySet, Bytecode, CodeUnit, FieldHandleIndex, FunctionDefinition,
        FunctionDefinitionIndex, FunctionHandle, Signature, SignatureIndex, SignatureToken,
        StructDefinition, StructDefinitionIndex, StructFieldInformation, StructTypeParameter,
        TableIndex, TypeSignature, Visibility,
    },
};
use dis::Instruction;
use move_bytecode_source_map::source_map::SourceMap;
use move_command_line_common::files::FileHash;
use move_compiler::compiled_unit::NamedCompiledModule;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_ir_types::location::Loc;
use move_symbol_pool::Symbol;

pub mod dis;
pub mod error;

pub fn translate<R: Read, W: Write>(mut from: R, mut to: W) -> Result<(), error::Error> {
    let mut buf = Default::default();
    let r_len = from.read_to_end(&mut buf)?;

    // TODO: fill me
    // NOTE: one code unit per function definition
    let mut code_unit = CodeUnit::default();
    // NOTE: then we can call `move_compiler::compiled_unit::verify_units()` and get some diagnostics.
    // The same for CompiledModule - verify_module(module.module) placed there: move-bytecode-verifier/src/verifier.rs

    // TODO: calc using op::writes_to_storage

    let ops = dis::read(buf.as_slice())?;
    for (offset, op) in ops.iter() {
        use dis::Instruction::*;

        println!("{:?}", op);

        // TODO: mop:Vec, because we have to produce many instructions for one sometimes.
        let mop = match op {
            Stop => todo!(),
            Add => Bytecode::Add,
            Mul => Bytecode::Mul,
            Sub => Bytecode::Sub,
            Div => Bytecode::Div,
            SDiv => todo!(),
            Mod => todo!(),
            SMod => todo!(),
            AddMod => todo!(),
            MulMod => todo!(),
            Exp => todo!(),
            SignExtend => todo!(),
            Lt => todo!(),
            Gt => todo!(),
            SLt => todo!(),
            SGt => todo!(),
            EQ => Bytecode::Eq,
            IsZero => todo!(),
            And => Bytecode::And,
            Or => Bytecode::Or,
            Xor => Bytecode::Xor,
            Not => Bytecode::Not,
            Byte => todo!(),
            Shl => todo!(),
            Shr => todo!(),
            Sar => todo!(),
            Sha3 => todo!(),
            Addr => todo!(),
            Balance => todo!(),
            Origin => todo!(),
            Caller => todo!(),
            CallValue => todo!(),
            CallDataLoad => todo!(),
            CallDataSize => todo!(),
            CallDataCopy => todo!(),
            CodeSize => todo!(),
            CodeCopy => todo!(),
            GasPrice => todo!(),
            ExtCodeSize => todo!(),
            ExtCodeCopy => todo!(),
            ReturnDataSize => todo!(),
            ReturnDataCopy => todo!(),
            ExtCodeHash => todo!(),
            Blockhash => todo!(),
            Coinbase => todo!(),
            Timestamp => todo!(),
            Number => todo!(),
            Difficulty => todo!(),
            GasLimit => todo!(),
            Pop => Bytecode::Pop,
            MLoad => todo!(),
            MStore => todo!(),
            MStore8 => todo!(),
            SLoad => todo!(),
            SStore => todo!(),
            Jump => todo!(),
            JumpIf => todo!("=> Bytecode::BrTrue"),
            JumpDest => todo!(),
            PC => todo!(),
            MSize => todo!(),
            Gas => todo!(),
            Push(bytes) => todo!(),
            Dup(value) => todo!(),
            Swap(pointer) => todo!(),
            Log(value) => todo!(),
            Create => todo!(),
            Create2 => todo!(),
            Call => todo!(),
            CallCode => todo!(),
            StaticCall => {
                Bytecode::Call(resolve_create_function_handle_index(*offset, &op, &ops))
            }
            DelegateCall => todo!(),
            Return => todo!(),
            Revert => todo!(),
            Invalid => todo!(),
            SelfDestruct => unimplemented!(),
        };
    }

    // TODO: create Module here
    let mut module = NamedCompiledModule {
        package_name: todo!(),
        address: todo!("address for this module, from config :NumericalAddress"),
        name: Symbol::from("name of this module, from Eth-contract"),
        module: CompiledModule {
            version: VERSION_MAX,
            // main
            self_module_handle_idx: ModuleHandleIndex::new(0),
            module_handles: vec![
                // self:
                ModuleHandle {
                    address: AddressIdentifierIndex::from(AddressIdentifierIndex(0)),
                    name: IdentifierIndex::from(IdentifierIndex(0)),
                },
            ],
            struct_handles: Default::default(),
            function_handles: Default::default(),
            field_handles: Default::default(),
            friend_decls: Default::default(),
            struct_def_instantiations: Default::default(),
				// entry points signatures:
				// TODO: take fn-signatures from ABI and/or heuristically from bytecode
            function_instantiations: Default::default(),
            field_instantiations: Default::default(),
            signatures: Default::default(),
            identifiers: vec![
                // self:
                Identifier::new("name of this module, from Eth-contract")?,
            ],
            address_identifiers: vec![
                // self
                AccountAddress::new(todo!(
                    "address for this module, from config :NumericalAddress"
                )),
            ],
            constant_pool: Default::default(),
            metadata: Default::default(),
            struct_defs: Default::default(),
            // TODO: bytecode bodies of functions
            function_defs: Default::default(),
        },
        source_map: SourceMap::new(Loc::new(FileHash::new("eht-contract.bin"), 0, 0), None),
    };

    Ok(())
}

fn resolve_create_function_handle_index(
    offset: usize,
    op: &Instruction,
    ops: &BTreeMap<usize, Instruction>,
) -> FunctionHandleIndex {
    todo!();
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC_HELLO_WORLD: &[u8] = include_bytes!(concat!("../", env!("HelloWorld")));
    const SRC_A_PLUS_B: &[u8] = include_bytes!(concat!("../", env!("APlusB")));

    #[test]
    #[ignore = "not ready yet"]
    fn translate_a_plus_b() -> Result<(), error::Error> {
        let mut buf = Vec::new();

        let _ = translate(SRC_A_PLUS_B, &mut buf)?;

        Ok(())
    }

    #[test]
    #[ignore = "so much complexity for now"]
    fn translate_hello_world() -> Result<(), error::Error> {
        let mut buf = Vec::new();

        let _ = translate(SRC_HELLO_WORLD, &mut buf)?;

        Ok(())
    }
}
