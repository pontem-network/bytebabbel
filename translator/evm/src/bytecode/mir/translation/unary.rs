use crate::bytecode::hir::executor::math::UnaryOp;
use crate::bytecode::hir::ir::var::VarId;
use crate::bytecode::mir::ir::math::Operation;
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::SType;
use crate::bytecode::mir::translation::consts::bool_const;
use crate::bytecode::mir::translation::Variable;
use crate::MirTranslator;
use anyhow::Error;
use std::rc::Rc;

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
            SType::Bool => self.unary_with_u128(cmd, var, result),
        }
    }

    fn unary_with_u128(
        &mut self,
        cmd: UnaryOp,
        op: Rc<Variable>,
        result: VarId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn unary_with_bool(
        &mut self,
        cmd: UnaryOp,
        op: Rc<Variable>,
        result: VarId,
    ) -> Result<(), Error> {
        match cmd {
            UnaryOp::IsZero => {
                let result = self.create_local_var(result, SType::Bool);
                let false_ = bool_const(false);
                let action = Statement::Operation(Operation::Eq, op, false_);
                self.mir
                    .add_statement(Statement::CreateVar(result, Box::new(action)));
            }
            UnaryOp::Not => {}
        }
        Ok(())
    }
}
