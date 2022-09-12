use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir2::ir::expression::Expr;
use crate::bytecode::instruction::Instruction;
use crate::BlockId;
use std::rc::Rc;

pub enum ControlFlow {
    Stop,
    Return,
    Revert,
    Abort(u8),
    Jump,
    JumpIf(Instruction),
}

impl InstructionHandler for ControlFlow {
    fn handle(&self, mut params: Vec<Rc<Expr>>, ctx: &mut Context) -> ExecutionResult {
        match self {
            ControlFlow::Stop => ExecutionResult::Stop,
            ControlFlow::Abort(code) => ExecutionResult::Abort(*code),
            ControlFlow::Return => {
                let len = params.remove(1);
                let offset = params.remove(0);
                ExecutionResult::Result { offset, len }
            }
            ControlFlow::Revert => ExecutionResult::Abort(255),
            ControlFlow::Jump => {
                let dest = params.remove(0);
                if let Some(block) = dest.resolve(ctx) {
                    ExecutionResult::Jmp(dest, BlockId::from(block.as_usize()))
                } else {
                    panic!("Unsupported dynamic jump");
                }
            }
            ControlFlow::JumpIf(inst) => {
                let cnd = params.remove(1);
                let true_br = params.remove(0);

                let true_br = true_br.resolve(ctx).expect("Unsupported dynamic jump if");
                let true_br = BlockId::from(true_br.as_usize());
                let false_br = BlockId::from(inst.next());

                if !ctx.is_in_loop() {
                    if let Some(cnd_val) = cnd.resolve(ctx) {
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
