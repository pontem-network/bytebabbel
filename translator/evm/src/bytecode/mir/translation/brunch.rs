use crate::bytecode::hir::ir::instruction::Instruction;
use crate::bytecode::hir::ir::var::{VarId, Vars};
use crate::bytecode::mir::ir::statement::Statement::StackOps;
use crate::bytecode::mir::ir::statement::{StackOpsBuilder, Statement, VarOrStack};
use crate::bytecode::mir::ir::types::SType;
use crate::bytecode::mir::ir::Mir;
use crate::bytecode::mir::translation::variables::Variables;
use crate::{BlockId, MirTranslator};
use anyhow::Error;

impl MirTranslator {
    pub fn translate_loop(
        &mut self,
        id: BlockId,
        condition_block: &[Instruction],
        condition: VarId,
        is_true_br_loop: bool,
        loop_br: &[Instruction],
        vars: &mut Vars,
    ) -> Result<(), Error> {
        let before = self.mir.swap(Mir::default());
        self.translate_instructions(condition_block, vars)?;
        let cnd_calc = self.mir.swap(Mir::default());
        self.translate_instructions(loop_br, vars)?;
        let loop_br = self.mir.swap(before);

        let cnd = self.use_var(condition)?;
        let cnd = if is_true_br_loop {
            // true branch
            let mut cnd_builder = StackOpsBuilder::default();
            cnd_builder.push(cnd);
            cnd_builder.not()?;
            VarOrStack::Stack(cnd_builder.build(SType::Bool)?)
        } else {
            // false branch
            VarOrStack::Var(cnd)
        };

        self.mir.add_statement(Statement::Loop {
            id,
            cnd_calc: cnd_calc.into_inner(),
            cnd,
            body: loop_br.into_inner(),
        });
        Ok(())
    }

    pub fn translate_if(
        &mut self,
        var: VarId,
        true_br: &[Instruction],
        false_br: &[Instruction],
        vars: &mut Vars,
    ) -> Result<(), Error> {
        let before = self.mir.swap(Mir::default());
        let cnd = self.use_var(var)?;

        self.translate_instructions(true_br, vars)?;
        let true_br = self.mir.swap(Mir::default());

        self.translate_instructions(false_br, vars)?;
        let false_br = self.mir.swap(before);

        self.mir.add_statement(Statement::IF {
            cnd: VarOrStack::Var(cnd),
            true_br: true_br.into_inner(),
            false_br: false_br.into_inner(),
        });

        Ok(())
    }

    pub fn translate_continue(
        &mut self,
        id: BlockId,
        true_br: &[Instruction],
        false_br: &[Instruction],
        vars: &mut Vars,
    ) {
    }
}
