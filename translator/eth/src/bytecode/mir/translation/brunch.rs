use anyhow::Error;

use crate::bytecode::hir::ir::VarId;
use crate::bytecode::hir::vars::Vars;
use crate::bytecode::mir::ir::expression::Expression;
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::SType;
use crate::bytecode::mir::ir::Mir;
use crate::{BlockId, MirTranslator};

impl<'a> MirTranslator<'a> {
    // pub fn translate_loop(
    //     &mut self,
    //     id: BlockId,
    //     condition_block: &[HirStmt],
    //     condition: VarId,
    //     is_true_br_loop: bool,
    //     loop_br: &[HirStmt],
    //     vars: &mut Vars,
    // ) -> Result<(), Error> {
    //     let before = self.mir.swap(Mir::default());
    //     self.translate_instructions(condition_block, vars)?;
    //     let cnd_calc = self.mir.swap(Mir::default());
    //     self.translate_instructions(loop_br, vars)?;
    //     let loop_br = self.mir.swap(before);
    //
    //     let cnd = self.get_var(condition)?;
    //     let cnd = self.cast(cnd, SType::Bool)?;
    //     let cnd = if is_true_br_loop {
    //         // true branch
    //         StackOpsBuilder::default()
    //             .push_bool(cnd)?
    //             .not()?
    //             .build(SType::Bool)?
    //     } else {
    //         // false branch
    //         *cnd.expr().expr
    //     };
    //
    //     self.mir.push(Statement::Loop {
    //         id,
    //         cnd_calc: cnd_calc.into_inner(),
    //         cnd,
    //         body: loop_br.into_inner(),
    //     });
    //     Ok(())
    // }
    //
    // pub fn translate_if(
    //     &mut self,
    //     var: VarId,
    //     true_br: &[HirStmt],
    //     false_br: &[HirStmt],
    //     vars: &mut Vars,
    // ) -> Result<(), Error> {
    //     let before = self.mir.swap(Mir::default());
    //     let cnd = self.get_var(var)?;
    //
    //     self.translate_instructions(true_br, vars)?;
    //     let true_br = self.mir.swap(Mir::default());
    //
    //     self.translate_instructions(false_br, vars)?;
    //     let false_br = self.mir.swap(before);
    //
    //     let cnd = self.cast(cnd, SType::Bool)?;
    //     self.mir.push(Statement::IF {
    //         cnd: Expression::MoveVar(cnd).ty(SType::Bool),
    //         true_br: true_br.into_inner(),
    //         false_br: false_br.into_inner(),
    //     });
    //
    //     Ok(())
    // }
}
