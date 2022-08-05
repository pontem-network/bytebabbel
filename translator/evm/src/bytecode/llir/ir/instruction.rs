use crate::bytecode::llir::ir::var::VarId;
use crate::U256;

#[derive(Debug)]
pub enum Instruction {
    SetVar(VarId),
    MemStore(U256, VarId),
}
