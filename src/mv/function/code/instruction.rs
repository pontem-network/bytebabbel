use crate::evm::bytecode::executor::stack::{MemCell, StackFrame};
use crate::evm::OpCode;
use anyhow::{bail, Error};
use bigint::U256;
use std::rc::Rc;

#[derive(Debug)]
pub enum InstractionTree {
    Inst2(OpCode, Box<InstractionTree>, Box<InstractionTree>),
    Const(U256),
    Args(u16),
    Addr,
}

impl InstractionTree {
    pub fn from(frame: &StackFrame) -> Result<InstractionTree, Error> {
        Ok(match frame.cell.as_ref() {
            MemCell::Val(val) => InstractionTree::Const(val.clone()),
            MemCell::Param(num) => InstractionTree::Args(*num),
            MemCell::NegativeStack => bail!("Unresolved negative stack"), //todo add loc data.
            MemCell::Unknown => bail!("Unresolved stack value"),
            MemCell::SelfAddress => InstractionTree::Addr,
            MemCell::Calc2(op_code, a, b) => InstractionTree::Inst2(
                op_code.clone(),
                Box::new(InstractionTree::from(a)?),
                Box::new(InstractionTree::from(a)?),
            ),
        })
    }
}
