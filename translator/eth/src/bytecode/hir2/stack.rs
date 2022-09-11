use crate::bytecode::hir2::ir::expression::Expr;
use std::rc::Rc;

#[derive(Default, Debug, Clone)]
pub struct Stack {
    pub stack: Vec<Rc<Expr>>,
}

impl Stack {
    pub fn push(&mut self, _to_push: Vec<Rc<Expr>>) {
        todo!()
    }

    pub fn pop(&mut self, _pops: usize) -> Vec<Rc<Expr>> {
        todo!()
    }
}
