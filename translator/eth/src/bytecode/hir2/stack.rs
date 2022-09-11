use crate::bytecode::hir2::ir::expression::Expr;
use std::rc::Rc;

#[derive(Default, Debug, Clone)]
pub struct Stack {
    pub stack: Vec<Rc<Expr>>,
}

impl Stack {
    pub fn push(&mut self, to_push: Vec<Rc<Expr>>) {
        self.stack.extend(to_push.into_iter().rev());
    }

    pub fn pop(&mut self, pops: usize) -> Vec<Rc<Expr>> {
        let mut res = Vec::with_capacity(pops);
        for _ in 0..pops {
            if let Some(item) = self.stack.pop() {
                res.push(item);
            }
        }
        res
    }
}
