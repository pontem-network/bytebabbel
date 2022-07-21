use crate::evm::bytecode::executor::execution::{Execution, FunctionFlow};
use crate::evm::bytecode::executor::stack::{Frame, StackFrame};
use crate::evm::function::FunDef;
use crate::evm::program::Program;
use crate::flog::is_trace;
use crate::mv::function::code::context::Context;
use crate::mv::function::code::intrinsic::math::{BinaryOpCode, MathModel, UnaryOpCode};
use crate::mv::function::code::writer::FunctionCode;
use crate::mv::function::signature::map_signature;
use anyhow::{anyhow, Error};
use move_binary_format::file_format::{Bytecode, SignatureToken};

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
        let input_types = map_signature(&def.abi.inputs);
        let output_types = map_signature(&def.abi.outputs);
        MvTranslator {
            program,
            def,
            ctx: Context::new(input_types, output_types),
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
        Ok(self.ctx.freeze())
    }

    fn translate_flow(&mut self, flow: &FunctionFlow) -> Result<(), Error> {
        for exec in flow.execution_tree() {
            match exec {
                Execution::SetVar(id, calc) => {
                    let token = self.map_calculation(calc);
                    self.ctx.set_global_var(*id, token);
                }
                Execution::Abort(code) => {
                    self.ctx.write_code(Bytecode::LdU64(*code as u64));
                    self.ctx.write_code(Bytecode::Abort);
                }
                Execution::Calc(frame) => {
                    self.map_calculation(frame);
                }
                Execution::Branch {
                    cnd,
                    true_br,
                    false_br,
                } => {
                    let tp = self.map_calculation(cnd);

                    if tp != SignatureToken::Bool {
                        self.math.write_to_bool(&mut self.ctx);
                    }

                    let start_false_br = self.ctx.pc();
                    self.ctx.write_code(Bytecode::BrTrue(0));
                    self.translate_flow(false_br)?;
                    let start_true_br = self.ctx.pc();
                    self.translate_flow(true_br)?;
                    self.ctx.update_jmp_pc(start_false_br, start_true_br as u16);
                }
            }
        }

        if !flow.result().is_empty() {
            for res in flow.result() {
                if !res.is_unit() {
                    self.ctx.st_var(res)?;
                }
            }

            self.ctx.write_code(Bytecode::Ret);
        }
        Ok(())
    }

    fn map_calculation(&mut self, calc: &StackFrame) -> SignatureToken {
        match calc.frame().as_ref() {
            Frame::Val(val) => self.math.set_literal(&mut self.ctx, val),
            Frame::Param(idx) => {
                self.ctx.write_code(Bytecode::CopyLoc(*idx as u8));
                if self.def.abi.inputs[*idx as usize].tp.as_str() == "bool" {
                    SignatureToken::Bool
                } else {
                    self.math.write_from_u128(&mut self.ctx)
                }
            }
            Frame::Bool(val) => {
                if *val {
                    self.ctx.write_code(Bytecode::LdTrue);
                } else {
                    self.ctx.write_code(Bytecode::LdFalse);
                }
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
                BinaryOpCode::code(self.math, &mut self.ctx, *code, a, b)
            }
            Frame::Abort(code) => {
                self.ctx.write_code(Bytecode::LdU64(*code));
                self.ctx.write_code(Bytecode::Abort);
                SignatureToken::U128
            }
            Frame::Calc(op, calc) => {
                let tp = self.map_calculation(calc);
                UnaryOpCode::code(self.math, &mut self.ctx, *op, tp)
            }
        }
    }
}
