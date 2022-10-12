use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir::ir::_Expr;
use crate::bytecode::instruction::Instruction;
use crate::bytecode::loc::Loc;
use crate::{Hir, Offset};

pub enum ControlFlow {
    Stop,
    Return,
    Revert,
    Abort(u8),
    Jump,
    JumpIf(Instruction),
}

impl InstructionHandler for ControlFlow {
    fn handle(
        &self,
        mut params: Vec<Loc<_Expr>>,
        ir: &mut Hir,
        ctx: &mut Context,
    ) -> ExecutionResult {
        match self {
            ControlFlow::Stop => {
                ir.stop(&ctx.loc);
                ExecutionResult::End
            }
            ControlFlow::Return => {
                let len = params.remove(1);
                let offset = params.remove(0);
                ir.return_(&ctx.loc, offset, len);
                ExecutionResult::End
            }
            ControlFlow::Revert => {
                ir.abort(&ctx.loc, 255);
                ExecutionResult::End
            }
            ControlFlow::Abort(code) => {
                ir.abort(&ctx.loc, *code);
                ExecutionResult::End
            }
            ControlFlow::Jump => {
                let dest = params.remove(0);
                let dest = dest
                    .resolve(ir, ctx)
                    .expect(&format!("Jump destination is not a constant. {:?}", dest));
                ExecutionResult::Jmp(Offset::from(dest))
            }
            ControlFlow::JumpIf(inst) => {
                let true_br = params
                    .remove(0)
                    .resolve(ir, ctx)
                    .expect("Unsupported dynamic jump if");
                let true_br = Offset::from(true_br);
                let false_br = Offset::from(inst.next());
                let cnd = params.remove(0);
                if !ctx.is_in_loop() {
                    if let Some(cnd_val) = cnd.resolve(ir, ctx) {
                        return ExecutionResult::Jmp(if cnd_val.is_zero() {
                            false_br
                        } else {
                            true_br
                        });
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
