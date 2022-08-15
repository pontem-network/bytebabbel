use crate::bytecode::hir::ir::instruction::Instruction;
use crate::bytecode::hir::ir::var::{VarId, Vars};
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::Mir;
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

        println!("{:?}", cnd_calc);
        println!("{:?}", loop_br);
        todo!()
    }
}
