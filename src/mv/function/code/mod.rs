use crate::evm::bytecode::executor::execution::{Execution, Var};
use crate::evm::bytecode::executor::stack::{Frame, StackFrame};
use crate::evm::function::FunctionDefinition;
use crate::evm::program::Program;
use anyhow::{anyhow, Error};
use move_binary_format::file_format::{Bytecode, SignatureToken};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct MvIr {
    locals: Vec<SignatureToken>,
    bytecode: Vec<Bytecode>,
}

impl MvIr {
    pub fn make_ir(def: FunctionDefinition, program: &Program) -> Result<MvIr, Error> {
        let flow = program
            .function_flow(def.hash)
            .ok_or_else(|| anyhow!("Root path for {} function not found.", def.abi.name))?;

        let mut bytecode = Vec::new();
        let mut locals = BTreeMap::new();

        for flow in flow.execution_tree() {
            match flow {
                Execution::SetVar(id, calc) => {
                    let token = map_calculation(calc, &mut bytecode)?;
                    locals.insert(*id, token);
                    move_to_var(*id, &mut bytecode)?;
                }
            }
        }

        for res in flow.result() {
            move_result(res, &mut bytecode)?;
        }

        bytecode.push(Bytecode::Ret);
        Ok(MvIr {
            locals: locals.into_iter().map(|(_, tkn)| tkn).collect(),
            bytecode,
        })
    }

    pub fn locals(&self) -> Vec<SignatureToken> {
        self.locals.clone()
    }

    pub fn bytecode(&self) -> Result<Vec<Bytecode>, Error> {
        Ok(self.bytecode.clone())
    }
}

fn move_result(var: &Var, bytecode: &mut Vec<Bytecode>) -> Result<(), Error> {
    bytecode.push(Bytecode::MoveLoc(var.index()));
    Ok(())
}

fn move_to_var(id: Var, bytecode: &mut Vec<Bytecode>) -> Result<(), Error> {
    bytecode.push(Bytecode::StLoc(id.index()));
    Ok(())
}

fn map_calculation(
    calc: &StackFrame,
    bytecode: &mut Vec<Bytecode>,
) -> Result<SignatureToken, Error> {
    match calc.frame().as_ref() {
        Frame::Val(val) => {
            bytecode.push(Bytecode::LdU128(val.as_u128()));
            Ok(SignatureToken::U128)
        }
        Frame::Param(_) => {
            todo!()
        }
        Frame::Bool(val) => {
            if *val {
                bytecode.push(Bytecode::LdTrue);
            } else {
                bytecode.push(Bytecode::LdFalse);
            }
            Ok(SignatureToken::Bool)
        }
        Frame::SelfAddress => {
            todo!()
        }
        Frame::Mem(_, _) => {
            todo!()
        }
        Frame::Calc2(_, _, _) => {
            todo!()
        }
        Frame::Abort => {
            todo!()
        }
        Frame::Calc(_, _) => {
            todo!()
        }
    }
}
