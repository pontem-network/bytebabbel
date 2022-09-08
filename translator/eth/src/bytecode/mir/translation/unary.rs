use crate::bytecode::hir::executor::math::UnaryOp;
use crate::bytecode::hir::ir::var::VarId;
use crate::bytecode::mir::ir::expression::StackOpsBuilder;
use crate::bytecode::mir::ir::math::Operation;
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::{SType, Value};
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
            SType::Number => self.unary_with_u128(cmd, var, result),
            SType::Bool => self.unary_with_bool(cmd, var, result),
            _ => Err(anyhow!(
                "Unary operation {:?} not supported for type {:?}",
                cmd,
                var.s_type()
            )),
        }
    }

    fn unary_with_u128(&mut self, cmd: UnaryOp, op: Variable, result: VarId) -> Result<(), Error> {
        let result = self.map_var(result, SType::Bool);
        match cmd {
            UnaryOp::IsZero => {
                let ops = StackOpsBuilder::default()
                    .push_var(op)
                    .push_const(Value::Number(0))
                    .binary_op(Operation::Eq, SType::Number, SType::Bool)?
                    .build(SType::Bool)?;
                self.mir.add_statement(Statement::CreateVar(result, ops));
            }
            UnaryOp::Not => {}
        }
        Ok(())
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

/*

entry public divede(Arg0: u128): u128 {
L0:     loc1: u128
L1:     loc2: u128
L2:     loc3: u128
L3:     loc4: u128
L4:     loc5: u128
B0:
        0: LdU128(128)
        1: StLoc[1](loc0: u128)
        2: CopyLoc[1](loc0: u128)
        3: StLoc[2](loc1: u128)
        4: CopyLoc[0](Arg0: u128)
        5: StLoc[3](loc2: u128)
        6: LdU128(10)
        7: StLoc[4](loc3: u128)
        8: CopyLoc[4](loc3: u128)
        9: LdU128(0)
        10: Eq
        11: BrTrue(13)
B1:
        12: CopyLoc[3](loc2: u128)
B2:
        13: CopyLoc[4](loc3: u128)
        14: Div
        15: StLoc[5](loc4: u128)
        16: Branch(18)
B3:
        17: LdU128(0)
B4:
        18: StLoc[5](loc4: u128)
        19: CopyLoc[5](loc4: u128)
        20: StLoc[6](loc5: u128)
        21: CopyLoc[5](loc4: u128)
        22: Ret
}
 */
