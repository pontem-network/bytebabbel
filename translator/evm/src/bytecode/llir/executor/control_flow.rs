use crate::bytecode::instruction::Instruction;
use crate::bytecode::llir::context::Context;
use crate::bytecode::llir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::llir::ir::var::VarId;
use crate::{BlockId, Ir};

pub enum ControlFlow {
    Stop,
    Return,
    Revert,
    Abort(u8),
    Jump,
    JumpIf(Instruction),
}

impl InstructionHandler for ControlFlow {
    fn handle(&self, params: Vec<VarId>, ir: &mut Ir, _: &mut Context) -> ExecutionResult {
        match self {
            ControlFlow::Stop => ExecutionResult::Stop,
            ControlFlow::Abort(code) => ExecutionResult::Abort(*code),
            ControlFlow::Return => ExecutionResult::Result {
                offset: params[0],
                len: params[1],
                revert: false,
            },
            ControlFlow::Revert => ExecutionResult::Result {
                offset: params[0],
                len: params[1],
                revert: true,
            },
            ControlFlow::Jump => {
                if let Some(block) = ir.resolve_var(params[0]) {
                    ExecutionResult::Jmp(BlockId::from(block.as_usize()))
                } else {
                    panic!("Unsupported dynamic jump");
                }
            }
            ControlFlow::JumpIf(inst) => {
                let true_br = ir
                    .resolve_var(params[0])
                    .expect("Unsupported dynamic jump if");
                let true_br = BlockId::from(true_br.as_usize());
                let false_br = BlockId::from(inst.next());
                let cnd = params[1];
                if let Some(cnd) = ir.resolve_var(cnd) {
                    ExecutionResult::Jmp(if cnd.is_zero() { false_br } else { true_br })
                } else {
                    ExecutionResult::CndJmp {
                        cnd,
                        true_br,
                        false_br,
                    }
                }
            }
        }
    }
}
