use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir::ir::var::VarId;
use crate::bytecode::instruction::Instruction;
use crate::{BlockId, Hir};

pub enum ControlFlow {
    Stop,
    Return,
    Revert,
    Abort(u8),
    Jump,
    JumpIf(Instruction),
}

impl InstructionHandler for ControlFlow {
    fn handle(&self, params: Vec<VarId>, ir: &mut Hir, ctx: &mut Context) -> ExecutionResult {
        match self {
            ControlFlow::Stop => ExecutionResult::Stop,
            ControlFlow::Abort(code) => ExecutionResult::Abort(*code),
            ControlFlow::Return => ExecutionResult::Result {
                offset: params[0],
                len: params[1],
            },
            ControlFlow::Revert => ExecutionResult::Abort(255),
            ControlFlow::Jump => {
                if let Some(block) = ir.resolve_var(params[0]) {
                    ExecutionResult::Jmp(params[0], BlockId::from(block.as_usize()))
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
                if !ctx.is_in_loop() {
                    if let Some(cnd_val) = ir.resolve_var(cnd) {
                        return ExecutionResult::Jmp(
                            cnd,
                            if cnd_val.is_zero() { false_br } else { true_br },
                        );
                    }
                }

                ExecutionResult::CndJmp {
                    cnd,
                    true_br,
                    false_br,
                }
            }
        }
    }
}
