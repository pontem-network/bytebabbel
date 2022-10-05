use crate::bytecode::hir2::ir::_Expr;
use crate::bytecode::loc::Loc;
use std::fmt::Debug;
use std::mem;

pub const FRAME_SIZE: usize = 32;

#[derive(Default, Clone)]
pub struct Stack {
    stack: Vec<Loc<_Expr>>,
}

impl Stack {
    pub fn clean(&mut self) {
        self.stack.clear();
    }

    pub fn pop(&mut self) -> Option<Loc<_Expr>> {
        self.stack.pop()
    }

    pub fn pop_vec(&mut self, count: usize) -> Vec<Loc<_Expr>> {
        let mut res = Vec::with_capacity(count);
        for _ in 0..count {
            if let Some(item) = self.stack.pop() {
                res.push(item);
            }
        }
        res
    }

    pub fn push(&mut self, push: Loc<_Expr>) {
        self.stack.push(push);
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Loc<_Expr>> {
        let idx = self.stack.len() - idx;
        self.stack.get_mut(idx)
    }

    pub fn dup(&mut self, idx: usize) {
        let item = self.stack[self.stack.len() - idx].clone();
        self.stack.push(item);
    }

    pub fn swap(&mut self, idx: usize) {
        let last_index = self.stack.len();
        self.stack.swap(last_index - idx, last_index - 1);
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn take(&mut self) -> Vec<Loc<_Expr>> {
        mem::take(&mut self.stack)
    }
}

impl Debug for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Stack {{")?;
        for (idx, expr) in self.stack.iter().enumerate().rev() {
            write!(f, " {} => {:?},", idx, expr)?;
        }
        write!(f, " }}")
    }
}
