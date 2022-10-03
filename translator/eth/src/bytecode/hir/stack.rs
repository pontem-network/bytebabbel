use std::fmt::Debug;

use crate::bytecode::hir::ir::var::VarId;

pub const FRAME_SIZE: usize = 32;

#[derive(Default, Debug, Clone)]
pub struct Stack {
    pub stack: Vec<VarId>,
}

impl Stack {
    pub fn clean(&mut self) {
        self.stack.clear();
    }

    pub fn pop(&mut self, count: usize) -> Vec<VarId> {
        let mut res = Vec::with_capacity(count);
        for _ in 0..count {
            if let Some(item) = self.stack.pop() {
                res.push(item);
            }
        }
        res
    }

    pub fn push(&mut self, to_push: Vec<VarId>) {
        self.stack.extend(to_push.into_iter().rev());
    }
}

// pub struct NegativeStack {
//     negative_stack: Vec<VarId>,
//     stack: Vec<VarId>,
// }
//
// impl NegativeStack {
//     pub fn pop(&mut self, count: usize) -> Vec<VarId> {
//         let mut res = Vec::with_capacity(count);
//
//         for _ in 0..count {
//             if let Some(item) = self.stack.pop() {
//                 res.push(item);
//             } else {
//                 self.negative_stack.push(VarId::from(0));
//             }
//         }
//         res
//     }
//
//     pub fn push(&mut self, to_push: Vec<VarId>) {
//         self.stack.extend(to_push.into_iter().rev());
//     }
// }
