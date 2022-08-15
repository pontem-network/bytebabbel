use crate::bytecode::hir::executor::math::UnaryOp;
use crate::bytecode::hir::ir::var::VarId;
use crate::bytecode::mir::ir::expression::StackOpsBuilder;
use crate::bytecode::mir::ir::math::Operation;
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::{SType, Value};
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
        let var = self.get_var(op)?;
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
        let result = self.map_var(result, SType::Bool);
        match cmd {
            UnaryOp::IsZero => {
                let ops = StackOpsBuilder::default()
                    .push_var(op)
                    .push_const(Value::from(false))
                    .binary_op(Operation::Eq, SType::Bool, SType::Bool)?
                    .build(SType::Bool)?;
                self.mir.add_statement(Statement::CreateVar(result, ops));
            }
            UnaryOp::Not => {
                let ops = StackOpsBuilder::default()
                    .push_var(op)
                    .not()?
                    .build(SType::Bool)?;
                self.mir.add_statement(Statement::CreateVar(result, ops));
            }
        }
        Ok(())
    }
}
