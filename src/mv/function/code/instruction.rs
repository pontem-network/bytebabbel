use crate::evm::bytecode::executor::stack::{MemCell, StackFrame};
use crate::evm::OpCode;
use anyhow::{anyhow, bail, Error};
use bigint::U256;

#[derive(Debug)]
pub enum InstructionTree {
    Inst2(Inst2, Box<InstructionTree>, Box<InstructionTree>),
    Const(U256),
    Args(u16),
    Var(U256, Box<InstructionTree>),
    Addr,
}

impl InstructionTree {
    pub fn from(frame: &StackFrame) -> Result<InstructionTree, Error> {
        Ok(match frame.cell.as_ref() {
            MemCell::Val(val) => InstructionTree::Const(val.clone()),
            MemCell::Param(num) => InstructionTree::Args(*num),
            MemCell::NegativeStack => bail!("Unresolved negative stack"), //todo add loc data.
            MemCell::Unknown => bail!("Unresolved stack value"),
            MemCell::SelfAddress => InstructionTree::Addr,
            MemCell::Calc2(op_code, a, b) => InstructionTree::Inst2(
                Inst2::try_from(op_code.clone())?,
                Box::new(InstructionTree::from(a)?),
                Box::new(InstructionTree::from(b)?),
            ),
            MemCell::Mem(rf, mem) => InstructionTree::Var(
                rf.as_u256()
                    .ok_or_else(|| anyhow!("Undefined mem address"))?,
                Box::new(InstructionTree::from(mem)?),
            ),
        })
    }
}

#[derive(Debug)]
pub enum Inst2 {
    Eq,
}

impl TryFrom<OpCode> for Inst2 {
    type Error = Error;

    fn try_from(value: OpCode) -> Result<Self, Self::Error> {
        Ok(match value {
            OpCode::EQ => Inst2::Eq,
            _ => bail!("Unsupported operation for Inst2. {value:?}"),
        })
    }
}
