use crate::bytecode::lir::ir::Expr;
use std::fmt::Debug;

pub const FRAME_SIZE: usize = 32;

#[derive(Default, Debug, Clone)]
pub struct Stack {
    pub stack: Vec<Expr>,
}

impl Stack {
    pub fn clean(&mut self) {
        self.stack.clear();
    }

    pub fn pop(&mut self, count: usize) -> Vec<Expr> {
        let mut res = Vec::with_capacity(count);
        for _ in 0..count {
            if let Some(item) = self.stack.pop() {
                res.push(item);
            }
        }
        res
    }

    pub fn push(&mut self, push: Expr) {
        self.stack.push(push);
    }

    pub fn dup(&mut self, idx: usize) {
        let item = self.stack[self.stack.len() - idx].clone();
        self.stack.push(item);
    }

    pub fn swap(&mut self, idx: usize) {
        let last_index = self.stack.len();
        self.stack.swap(last_index - idx, last_index - 1);
    }
}

#[cfg(test)]
mod tests {
    use crate::bytecode::lir::ir::Expr;
    use crate::bytecode::lir::stack::Stack;
    use crate::OpCode;

    #[test]
    pub fn test_dub() {
        let mut stack = Stack::default();
        stack.push(0.into());
        stack.push(1.into());
        stack.push(2.into());
        stack.push(3.into());
        stack.push(4.into());
        stack.push(5.into());
        stack.push(6.into());
        stack.push(7.into());
        stack.push(8.into());
        stack.push(9.into());
        stack.push(10.into());

        stack.dup(OpCode::Dup(11).pops());
        stack.dup(OpCode::Dup(11).pops());
        stack.dup(OpCode::Dup(11).pops());
        stack.dup(OpCode::Dup(11).pops());
        stack.dup(OpCode::Dup(11).pops());
        stack.dup(OpCode::Dup(11).pops());
        stack.dup(OpCode::Dup(11).pops());
        stack.dup(OpCode::Dup(11).pops());
        stack.dup(OpCode::Dup(11).pops());
        stack.dup(OpCode::Dup(11).pops());
        stack.dup(OpCode::Dup(11).pops());

        let expected = vec![
            10.into(),
            9.into(),
            8.into(),
            7.into(),
            6.into(),
            5.into(),
            4.into(),
            3.into(),
            2.into(),
            1.into(),
            0.into(),
        ];
        assert_eq!(stack.pop(11), expected);
        assert_eq!(stack.pop(11), expected);

        stack.push(0.into());
        stack.dup(OpCode::Dup(1).pops());
        let expected = vec![0.into(), 0.into()];
        assert_eq!(stack.pop(2), expected);

        stack.push(0.into());
        stack.push(1.into());
        stack.push(2.into());
        stack.dup(OpCode::Dup(2).pops());

        let expected = vec![1.into(), 2.into(), 1.into(), 0.into()];
        assert_eq!(stack.pop(4), expected);
    }

    #[test]
    pub fn test_swap() {
        let mut stack = Stack::default();
        stack.push(0.into());
        stack.push(1.into());
        stack.push(2.into());
        stack.push(3.into());
        stack.push(4.into());
        stack.push(5.into());
        stack.push(6.into());
        stack.push(7.into());
        stack.push(8.into());
        stack.push(9.into());
        stack.push(10.into());

        stack.swap(OpCode::Swap(1).pops());
        let expected = vec![
            9.into(),
            10.into(),
            8.into(),
            7.into(),
            6.into(),
            5.into(),
            4.into(),
            3.into(),
            2.into(),
            1.into(),
            0.into(),
        ];
        assert_eq!(stack.pop(11), expected);

        stack.push(0.into());
        stack.push(1.into());
        stack.push(2.into());
        stack.push(3.into());
        stack.push(4.into());
        stack.push(5.into());
        stack.push(6.into());
        stack.push(7.into());
        stack.push(8.into());
        stack.push(9.into());
        stack.push(10.into());

        stack.swap(OpCode::Swap(2).pops());

        let expected = vec![
            8.into(),
            9.into(),
            10.into(),
            7.into(),
            6.into(),
            5.into(),
            4.into(),
            3.into(),
            2.into(),
            1.into(),
            0.into(),
        ];
        assert_eq!(stack.pop(11), expected);

        stack.push(0.into());
        stack.push(1.into());
        stack.push(2.into());
        stack.push(3.into());
        stack.push(4.into());
        stack.push(5.into());
        stack.push(6.into());
        stack.push(7.into());
        stack.push(8.into());
        stack.push(9.into());
        stack.push(10.into());

        stack.swap(OpCode::Swap(10).pops());
        let expected = vec![
            0.into(),
            9.into(),
            8.into(),
            7.into(),
            6.into(),
            5.into(),
            4.into(),
            3.into(),
            2.into(),
            1.into(),
            10.into(),
        ];
        assert_eq!(stack.pop(11), expected);
    }
}
