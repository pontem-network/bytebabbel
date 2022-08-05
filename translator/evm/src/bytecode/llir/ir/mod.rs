use crate::bytecode::llir::ir::debug::print_ir;
use crate::bytecode::llir::ir::instruction::Instruction;
use crate::bytecode::llir::ir::var::{Var, VarId, Vars};

mod debug;
mod instruction;
pub mod var;

#[derive(Default, Debug)]
pub struct Ir {
    vars: Vars,
    instructions: Vec<Instruction>,
}

impl Ir {
    pub fn create_var(&mut self, var: Var) -> VarId {
        let id = self.vars.create(var);
        self.instructions.push(Instruction::SetVar(id));
        id
    }

    pub fn print(&self) {
        print_ir(self);
    }
}
