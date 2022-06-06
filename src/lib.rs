use std::collections::BTreeMap;
use std::io::Read;
use std::io::Write;

use move_binary_format::file_format::FunctionHandleIndex;
use move_binary_format::{binary_views::BinaryIndexedView,
                     control_flow_graph::{ControlFlowGraph, VMControlFlowGraph},
                     file_format::{Ability, AbilitySet, Bytecode, CodeUnit, FieldHandleIndex, FunctionDefinition, FunctionDefinitionIndex, FunctionHandle,
                                   Signature, SignatureIndex, SignatureToken, StructDefinition, StructDefinitionIndex, StructFieldInformation,
                                   StructTypeParameter, TableIndex, TypeSignature, Visibility}};
use dis::Instruction;


pub mod error;
pub mod dis;


pub fn translate<R: Read, W: Write>(mut from: R, mut to: W) -> Result<(), error::Error> {
	let mut buf = Default::default();
	let r_len = from.read_to_end(&mut buf)?;

	// TODO: create Module here

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
			StaticCall => Bytecode::Call(resolve_create_function_handle_index(*offset, &op, &ops)),
			DelegateCall => todo!(),
			Return => todo!(),
			Revert => todo!(),
			Invalid => todo!(),
			SelfDestruct => unimplemented!(),
		};
	}


	Ok(())
}


fn resolve_create_function_handle_index(offset: usize, op: &Instruction, ops: &BTreeMap<usize, Instruction>) -> FunctionHandleIndex {
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
