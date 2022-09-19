use crate::bytecode::hir2::ir::expression::Expr;

#[derive(Default, Debug, Clone)]
pub struct Stack {
    pub stack: Vec<Expr>,
}

impl Stack {
    pub fn push(&mut self, to_push: Vec<Expr>) {
        self.stack.extend(to_push.into_iter().rev());
    }

    pub fn pop(&mut self, pops: usize) -> Vec<Expr> {
        let mut res = Vec::with_capacity(pops);
        for _ in 0..pops {
            if let Some(item) = self.stack.pop() {
                res.push(item);
            }
        }
        res
    }
}
