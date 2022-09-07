use crate::bytecode::hir::executor::math::TernaryOp;
use crate::bytecode::hir::ir::var::VarId;
use crate::MirTranslator;
use anyhow::Error;

impl<'a> MirTranslator<'a> {
    pub(super) fn translate_ternary_op(
        &mut self,
        _cmd: TernaryOp,
        op: VarId,
        op1: VarId,
        op2: VarId,
        _result: VarId,
    ) -> Result<(), Error> {
        let _op = self.get_var(op)?;
        let _op1 = self.get_var(op1)?;
        let _op2 = self.get_var(op2)?;
        todo!()
    }
}
