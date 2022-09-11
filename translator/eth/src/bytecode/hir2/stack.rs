use crate::bytecode::hir2::ir::expression::Expr;

#[derive(Default, Debug, Clone)]
pub struct Stack {}

impl Stack {
    pub fn push(&mut self, _to_push: Vec<Expr>) {
        todo!()
    }

    pub fn pop(&mut self, _pops: usize) -> Vec<Expr> {
        todo!()
    }
}
