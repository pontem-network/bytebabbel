use crate::bytecode::hir::executor::math::BinaryOp;
use crate::bytecode::hir::ir::var::VarId;
use crate::bytecode::mir::ir::math::Operation;
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::SType;
use crate::MirTranslator;
use anyhow::Error;

impl<'a> MirTranslator<'a> {
    pub(super) fn translate_binary_op(
        &mut self,
        cmd: BinaryOp,
        op: VarId,
        op1: VarId,
        result: VarId,
    ) -> Result<(), Error> {
        let op = self.get_var(op)?;
        let op1 = self.get_var(op1)?;

        let (op, op1) = if cmd == BinaryOp::EQ {
            if op.s_type() == SType::Bool && op1.s_type() == SType::Bool {
                (op, op1)
            } else {
                (self.cast(op, SType::Num)?, self.cast(op1, SType::Num)?)
            }
        } else {
            let op = self.cast(op, SType::Num)?;
            let op1 = self.cast(op1, SType::Num)?;
            (op, op1)
        };

        let cmd: Operation = cmd.into();
        let result = match cmd {
            Operation::Eq | Operation::Lt | Operation::Gt | Operation::SLt | Operation::SGt => {
                self.map_var(result, SType::Bool)
            }
            _ => self.map_var(result, SType::Num),
        };
        self.mir
            .add_statement(Statement::CreateVar(result, cmd.expr(op, op1)));
        Ok(())
    }
}
