use crate::bytecode::llir::ir::var::VarId;
use crate::U256;

#[derive(Debug)]
pub enum Instruction {
    SetVar(VarId),
    MemStore(U256, VarId),
    Branch {
        condition: VarId,
        true_branch_len: u64,
        false_branch_len: u64,
    },
}
