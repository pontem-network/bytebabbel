use crate::bytecode::hir::executor::math::UnaryOp;
use crate::bytecode::hir::ir::var::VarId;
use crate::bytecode::mir::ir::expression::StackOpsBuilder;
use crate::bytecode::mir::ir::math::Operation;
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::SType;
use crate::bytecode::mir::translation::Variable;
use crate::MirTranslator;
use anyhow::{anyhow, Error};

impl<'a> MirTranslator<'a> {
    pub(super) fn translate_unary_op(
        &mut self,
        cmd: UnaryOp,
        op: VarId,
        result: VarId,
    ) -> Result<(), Error> {
        let var = self.get_var(op)?;
        match var.s_type() {
            SType::Num => self.unary_with_num(cmd, var, result),
            SType::Bool => self.unary_with_bool(cmd, var, result),
            _ => Err(anyhow!(
                "Unary operation {:?} not supported for type {:?}",
                cmd,
                var.s_type()
            )),
        }
    }

    fn unary_with_num(&mut self, cmd: UnaryOp, op: Variable, result: VarId) -> Result<(), Error> {
        let result = self.map_var(result, SType::Bool);
        match cmd {
            UnaryOp::IsZero => self.mir.add_statement(Statement::CreateVar(
                result,
                Operation::IsZero.expr(op, Variable::none()),
            )),
            UnaryOp::Not => self.mir.add_statement(Statement::CreateVar(
                result,
                Operation::BitNot.expr(op, Variable::none()),
            )),
        }
        Ok(())
    }

    fn unary_with_bool(&mut self, cmd: UnaryOp, op: Variable, result: VarId) -> Result<(), Error> {
        let result = self.map_var(result, SType::Bool);
        match cmd {
            UnaryOp::IsZero => {
                let ops = StackOpsBuilder::default()
                    .push_bool(op)?
                    .push_const_bool(false)
                    .eq()?
                    .build(SType::Bool)?;
                self.mir.add_statement(Statement::CreateVar(result, ops));
            }
            UnaryOp::Not => {
                let ops = StackOpsBuilder::default()
                    .push_bool(op)?
                    .not()?
                    .build(SType::Bool)?;
                self.mir.add_statement(Statement::CreateVar(result, ops));
            }
        }
        Ok(())
    }
}
