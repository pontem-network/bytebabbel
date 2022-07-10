use crate::evm::bytecode::executor::debug::print_flow;
use crate::evm::bytecode::executor::execution::{Execution, FunctionFlow, Var};
use crate::evm::bytecode::executor::ops::UnaryOp;
use crate::evm::bytecode::executor::stack::{Frame, StackFrame};
use crate::evm::bytecode::executor::types::U256;
use crate::evm::function::FunDef;
use crate::evm::program::Program;
use crate::mv::function::code::intrinsic::{is_zero_bool, is_zero_uint};
use crate::mv::function::code::ops::IntoCode;
use anyhow::{anyhow, Error};
use move_binary_format::file_format::{Bytecode, SignatureToken};
use std::collections::BTreeMap;
use std::mem;

pub mod intrinsic;
pub mod ops;

const DEFAULT_WIDTH: usize = 4;

pub struct MvTranslator<'a> {
    program: &'a Program,
    locals: BTreeMap<Var, SignatureToken>,
    bytecode: Vec<Bytecode>,
    trace: bool,
    width: usize,
}

impl<'a> MvTranslator<'a> {
    pub fn new(program: &'a Program) -> MvTranslator<'a> {
        MvTranslator {
            program,
            locals: Default::default(),
            bytecode: vec![],
            trace: program.trace,
            width: DEFAULT_WIDTH,
        }
    }

    pub fn translate_fun(
        &mut self,
        def: &FunDef,
    ) -> Result<(Vec<SignatureToken>, Vec<Bytecode>), Error> {
        let flow = self
            .program
            .function_flow(def.hash)
            .ok_or_else(|| anyhow!("Root path for {} function not found.", def.abi.name))?;
        if self.trace {
            println!("flow:");
            print_flow(flow, self.width);
            println!();
        }
        self.translate_flow(flow, def)?;
        Ok((
            self.locals.iter().map(|(_, v)| v.clone()).collect(),
            mem::take(&mut self.bytecode),
        ))
    }

    fn pc(&self) -> usize {
        self.bytecode.len()
    }

    fn push(&mut self, bytecode: Bytecode) {
        if self.trace {
            print!("{}:", self.bytecode.len());
            println!("{:.<width$}{bytecode:?}", ".", width = self.width);
        }
        self.bytecode.push(bytecode);
    }

    fn extend<I: IntoIterator<Item = Bytecode>>(&mut self, bytecode: I) {
        for op in bytecode.into_iter() {
            self.push(op);
        }
    }

    fn translate_flow(&mut self, flow: &FunctionFlow, def: &FunDef) -> Result<(), Error> {
        for exec in flow.execution_tree() {
            match exec {
                Execution::SetVar(id, calc) => {
                    let token = self.map_calculation(calc, def)?;
                    self.locals.insert(*id, token);
                    self.move_to_var(*id, def);
                }
                Execution::Abort(code) => {
                    self.push(Bytecode::LdU64(*code as u64));
                    self.push(Bytecode::Abort);
                }
                Execution::Calc(frame) => {
                    self.map_calculation(frame, def)?;
                }
                Execution::Branch { true_br, false_br } => {
                    let start_false_br = self.pc();
                    self.push(Bytecode::BrTrue(0));
                    self.width += DEFAULT_WIDTH;
                    self.translate_flow(false_br, def)?;
                    let start_true_br = self.pc();
                    self.translate_flow(true_br, def)?;
                    self.width -= DEFAULT_WIDTH;
                    self.bytecode[start_false_br] = Bytecode::BrTrue(start_true_br as u16);
                }
            }
        }

        if !flow.result().is_empty() {
            for res in flow.result() {
                self.move_result(res, def);
            }
            self.push(Bytecode::Ret);
        }
        Ok(())
    }

    fn map_calculation(
        &mut self,
        calc: &StackFrame,
        def: &FunDef,
    ) -> Result<SignatureToken, Error> {
        match calc.frame().as_ref() {
            Frame::Val(val) => {
                if val > &U256::from(u128::MAX) {
                    // ¯\_(ツ)_/¯ todo replace u128 with u256
                    self.push(Bytecode::LdU128(u128::MAX));
                } else {
                    self.push(Bytecode::LdU128(val.as_u128()));
                }
                Ok(SignatureToken::U128)
            }
            Frame::Param(idx) => {
                self.push(Bytecode::CopyLoc(*idx as u8));
                if def.abi.inputs[*idx as usize].tp.as_str() == "bool" {
                    Ok(SignatureToken::Bool)
                } else {
                    Ok(SignatureToken::U128)
                }
            }
            Frame::Bool(val) => {
                if *val {
                    self.push(Bytecode::LdTrue);
                } else {
                    self.push(Bytecode::LdFalse);
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
                self.map_calculation(first, def)?;
                self.map_calculation(second, def)?;
                self.extend(code.bytecode());
                Ok(code.signature_type())
            }
            Frame::Abort(code) => {
                self.push(Bytecode::LdU64(*code));
                self.push(Bytecode::Abort);
                Ok(SignatureToken::U128)
            }
            Frame::Calc(op, calc) => {
                let tp = self.map_calculation(calc, def)?;
                match op {
                    UnaryOp::IsZero => {
                        if tp.is_integer() {
                            self.extend(is_zero_uint());
                        } else {
                            self.extend(is_zero_bool());
                        }
                    }
                    UnaryOp::Not => {
                        if tp.is_integer() {
                            todo!("may by unsupported")
                        } else {
                            self.push(Bytecode::Neq);
                        }
                    }
                }
                Ok(SignatureToken::Bool)
            }
        }
    }

    fn move_to_var(&mut self, id: Var, def: &FunDef) {
        self.push(Bytecode::StLoc(id.index() + def.abi.inputs.len() as u8));
    }

    fn move_result(&mut self, var: &Var, def: &FunDef) {
        self.push(Bytecode::MoveLoc(var.index() + def.abi.inputs.len() as u8));
    }
}
