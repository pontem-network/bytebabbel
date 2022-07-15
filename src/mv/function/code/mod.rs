use crate::evm::bytecode::executor::debug::print_flow;
use crate::evm::bytecode::executor::execution::{Execution, FunctionFlow, Var};
use crate::evm::bytecode::executor::ops::UnaryOp;
use crate::evm::bytecode::executor::stack::{Frame, StackFrame};
use crate::evm::bytecode::executor::types::U256;
use crate::evm::function::FunDef;
use crate::evm::program::Program;
use crate::mv::function::code::intrinsic::math::MathModel;
use crate::mv::function::code::intrinsic::{is_zero_bool, is_zero_uint};
use crate::mv::function::code::ops::IntoCode;
use crate::mv::function::code::writer::{CodeWriter, FunctionCode};
use anyhow::{anyhow, Error};
use move_binary_format::file_format::{Bytecode, LocalIndex, SignatureToken};
use std::collections::HashMap;

pub mod intrinsic;
pub mod ops;
pub mod writer;

pub struct MvTranslator<'a, M: MathModel> {
    program: &'a Program,
    def: &'a FunDef<'a>,
    code: CodeWriter,
    local_mapping: HashMap<Var, LocalIndex>,
    _math: &'a mut M,
}

impl<'a, M: MathModel> MvTranslator<'a, M> {
    pub fn new(program: &'a Program, def: &'a FunDef, math: &'a mut M) -> MvTranslator<'a, M> {
        MvTranslator {
            program,
            def,
            code: CodeWriter::new(def.abi.inputs.len(), program.trace),
            local_mapping: Default::default(),
            _math: math,
        }
    }

    pub fn translate(mut self) -> Result<FunctionCode, Error> {
        let flow = self
            .program
            .function_flow(self.def.hash)
            .ok_or_else(|| anyhow!("Root path for {} function not found.", self.def.abi.name))?;
        if self.program.trace {
            println!("flow:");
            print_flow(flow, 4);
            println!();
        }
        self.translate_flow(flow)?;
        Ok(self.code.freeze())
    }

    fn translate_flow(&mut self, flow: &FunctionFlow) -> Result<(), Error> {
        for exec in flow.execution_tree() {
            match exec {
                Execution::SetVar(id, calc) => {
                    let token = self.map_calculation(calc)?;
                    let idx = self.code.set_var(token);
                    self.local_mapping.insert(*id, idx);
                }
                Execution::Abort(code) => {
                    self.code.push(Bytecode::LdU64(*code as u64));
                    self.code.push(Bytecode::Abort);
                }
                Execution::Calc(frame) => {
                    self.map_calculation(frame)?;
                }
                Execution::Branch { true_br, false_br } => {
                    let start_false_br = self.code.pc();
                    self.code.push(Bytecode::BrTrue(0));
                    self.translate_flow(false_br)?;
                    let start_true_br = self.code.pc();
                    self.translate_flow(true_br)?;
                    self.code
                        .set_op(start_false_br, Bytecode::BrTrue(start_true_br as u16));
                }
            }
        }

        if !flow.result().is_empty() {
            for res in flow.result() {
                let local = self
                    .local_mapping
                    .get(res)
                    .ok_or_else(|| anyhow!("Unknown result variable:{}", res))?;
                self.code.move_local(*local);
            }
            self.code.push(Bytecode::Ret);
        }
        Ok(())
    }

    fn map_calculation(&mut self, calc: &StackFrame) -> Result<SignatureToken, Error> {
        match calc.frame().as_ref() {
            Frame::Val(val) => {
                if val > &U256::from(u128::MAX) {
                    // ¯\_(ツ)_/¯ todo replace u128 with u256
                    self.code.push(Bytecode::LdU128(u128::MAX));
                } else {
                    self.code.push(Bytecode::LdU128(val.as_u128()));
                }
                Ok(SignatureToken::U128)
            }
            Frame::Param(idx) => {
                self.code.push(Bytecode::CopyLoc(*idx as u8));
                if self.def.abi.inputs[*idx as usize].tp.as_str() == "bool" {
                    Ok(SignatureToken::Bool)
                } else {
                    Ok(SignatureToken::U128)
                }
            }
            Frame::Bool(val) => {
                if *val {
                    self.code.push(Bytecode::LdTrue);
                } else {
                    self.code.push(Bytecode::LdFalse);
                }
                Ok(SignatureToken::Bool)
            }
            Frame::SelfAddress => {
                todo!()
            }
            Frame::Mem(_, _) => {
                todo!()
            }
            Frame::Calc2(code, first, second) => {
                self.map_calculation(first)?;
                self.map_calculation(second)?;
                self.code.extend(code.bytecode());
                Ok(code.signature_type())
            }
            Frame::Abort(code) => {
                self.code.push(Bytecode::LdU64(*code));
                self.code.push(Bytecode::Abort);
                Ok(SignatureToken::U128)
            }
            Frame::Calc(op, calc) => {
                let tp = self.map_calculation(calc)?;
                match op {
                    UnaryOp::IsZero => {
                        if tp.is_integer() {
                            self.code.extend(is_zero_uint());
                        } else {
                            self.code.extend(is_zero_bool());
                        }
                    }
                    UnaryOp::Not => {
                        if tp.is_integer() {
                            todo!("may by unsupported")
                        } else {
                            self.code.push(Bytecode::Neq);
                        }
                    }
                }
                Ok(SignatureToken::Bool)
            }
        }
    }
}
