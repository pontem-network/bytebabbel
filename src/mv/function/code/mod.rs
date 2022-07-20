use crate::evm::bytecode::executor::execution::{Execution, FunctionFlow, Var};
use crate::evm::bytecode::executor::stack::{Frame, StackFrame};
use crate::evm::function::FunDef;
use crate::evm::program::Program;
use crate::mv::function::code::context::Context;
use crate::mv::function::code::intrinsic::math::{BinaryOpCode, MathModel, UnaryOpCode};
use crate::mv::function::code::stack::Stack;
use crate::mv::function::code::writer::{CodeWriter, FunctionCode};
use anyhow::{anyhow, Error};
use move_binary_format::file_format::{Bytecode, LocalIndex, SignatureToken};
use std::collections::HashMap;

pub mod context;
pub mod intrinsic;
pub mod ops;
pub mod stack;
pub mod writer;

pub struct MvTranslator<'a, M: MathModel> {
    program: &'a Program,
    def: &'a FunDef<'a>,
    ctx: Context,
    math: &'a mut M,
}

impl<'a, M: MathModel> MvTranslator<'a, M> {
    pub fn new(program: &'a Program, def: &'a FunDef, math: &'a mut M) -> MvTranslator<'a, M> {
        MvTranslator {
            program,
            def,
            ctx: Context::new(CodeWriter::new(def.abi.inputs.len())),
            math,
        }
    }

    pub fn translate(mut self) -> Result<FunctionCode, Error> {
        let flow = self
            .program
            .function_flow(self.def.hash)
            .ok_or_else(|| anyhow!("Root path for {} function not found.", self.def.abi.name))?;

        if is_trace() {
            log::trace!("\n{}", &self.program.debug_fn_by_hash(self.def.hash));
            log::trace!("{:?}\n", flow);
        }
        self.translate_flow(flow)?;
        Ok(self.code.freeze())
    }

    fn translate_flow(&mut self, flow: &FunctionFlow) -> Result<(), Error> {
        for exec in flow.execution_tree() {
            match exec {
                Execution::SetVar(id, calc) => {
                    let token = self.map_calculation(calc)?;
                    self.ctx.set_global_var(*id, token);
                }
                Execution::Abort(code) => {
                    self.ctx.write_code(Bytecode::LdU64(*code as u64));
                    self.ctx.write_code(Bytecode::Abort);
                }
                Execution::Calc(frame) => {
                    self.map_calculation(frame)?;
                }
                Execution::Branch {
                    cnd,
                    true_br,
                    false_br,
                } => {
                    let tp = self.map_calculation(cnd)?;

                    if tp != SignatureToken::Bool {
                        self.math.write_to_bool(&mut self.code);
                    }

                    let start_false_br = self.code.pc();
                    self.ctx.write_code(Bytecode::BrTrue(0));
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
                if !res.is_unit() {
                    let local = self
                        .local_mapping
                        .get(res)
                        .ok_or_else(|| anyhow!("Unknown result variable:{}", res))?;
                    self.code.move_local(*local);
                }
            }
            self.ctx.write_code(Bytecode::Ret);
        }
        Ok(())
    }

    fn map_calculation(&mut self, calc: &StackFrame) -> SignatureToken {
        match calc.frame().as_ref() {
            Frame::Val(val) => {
                let tp = self.math.set_literal(&mut self.code, val);
                self.stack.push(tp.clone());
                tp
            }
            Frame::Param(idx) => {
                self.ctx.write_code(Bytecode::CopyLoc(*idx as u8));
                if self.def.abi.inputs[*idx as usize].tp.as_str() == "bool" {
                    self.stack.push(SignatureToken::Bool);
                    SignatureToken::Bool
                } else {
                    let tp = self.math.write_from_u128(&mut self.code);
                    self.stack.push(tp);
                }
            }
            Frame::Bool(val) => {
                if *val {
                    self.ctx.write_code(Bytecode::LdTrue);
                } else {
                    self.ctx.write_code(Bytecode::LdFalse);
                }
                self.stack.push(SignatureToken::Bool);
                SignatureToken::Bool
            }
            Frame::SelfAddress => {
                todo!()
            }
            Frame::Mem(_, _) => {
                todo!()
            }
            Frame::Calc2(code, first, second) => {
                let a = self.map_calculation(first);
                let b = self.map_calculation(second);
                let tp = BinaryOpCode::code(self.math, &mut self.code, *code, a, b);
                self.stack.pop2();
                self.stack.push(tp.clone());
                tp
            }
            Frame::Abort(code) => {
                self.ctx.write_code(Bytecode::LdU64(*code));
                self.ctx.write_code(Bytecode::Abort);
                SignatureToken::U128
            }
            Frame::Calc(op, calc) => {
                let tp = self.map_calculation(calc);
                let tp = UnaryOpCode::code(self.math, &mut self.code, *op, tp);
                self.stack.pop();
                self.stack.push(tp.clone());
                tp
            }
        }
    }
}
