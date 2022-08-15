use crate::bytecode::hir::executor::math::UnaryOp;
use crate::bytecode::hir::ir::var::VarId;
use crate::bytecode::mir::ir::math::Operation;
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::SType;
use crate::bytecode::mir::translation::consts::bool_const;
use crate::bytecode::mir::translation::Variable;
use crate::MirTranslator;
use anyhow::Error;

impl MirTranslator {
    pub(super) fn translate_unary_op(
        &mut self,
        cmd: UnaryOp,
        op: VarId,
        result: VarId,
    ) -> Result<(), Error> {
        let var = self.use_var(op)?;
        match var.s_type() {
            SType::U128 => self.unary_with_u128(cmd, var, result),
            SType::Bool => self.unary_with_bool(cmd, var, result),
        }
    }

    fn unary_with_u128(
        &mut self,
        _cmd: UnaryOp,
        _op: Variable,
        _result: VarId,
    ) -> Result<(), Error> {
        todo!()
    }

    fn unary_with_bool(&mut self, cmd: UnaryOp, op: Variable, result: VarId) -> Result<(), Error> {
        match cmd {
            UnaryOp::IsZero => {
                let result = self.map_local_var(result, SType::Bool);
                let false_ = bool_const(false);
                let action = Statement::Operation(Operation::Eq, op, false_);
                self.mir
                    .add_statement(Statement::CreateVar(result, Box::new(action)));
            }
            UnaryOp::Not => {
                let result = self.map_local_var(result, SType::Bool);
                let action = Statement::Operation(Operation::Not, op.clone(), op);
                self.mir
                    .add_statement(Statement::CreateVar(result, Box::new(action)));
            }
        }
        Ok(())
    }
}
