use crate::bytecode::llir::ir::var::VarId;

#[derive(Debug)]
pub enum Instruction {
    SetVar(VarId),
}
