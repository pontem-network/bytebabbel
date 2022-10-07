use crate::bytecode::loc::Loc;
use crate::bytecode::mir::ir::expression::{Expression, TypedExpr};
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::{LocalIndex, SType};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Variables {
    locals: HashMap<SType, Locals>,
    seq: LocalIndex,
    list: Vec<SType>,
    input: Vec<SType>,
}

impl Variables {
    pub fn new(params: Vec<SType>) -> Variables {
        Variables {
            locals: HashMap::new(),
            seq: params.len() as LocalIndex,
            list: vec![],
            input: params,
        }
    }

    pub fn borrow(&mut self, tp: SType) -> Variable {
        let idx = self.seq;
        let locals = self.locals.entry(tp).or_default();
        if let Some(idx) = locals.borrow() {
            Variable(idx, tp)
        } else {
            locals.new_borrowed(idx);
            self.seq += 1;
            self.list.push(tp);
            Variable(idx, tp)
        }
    }

    pub fn reborrow(&mut self, var: Variable) {
        let idx = self.seq;
        let locals = self.locals.entry(var.1).or_default();
        locals.borrow_with_id(idx);
    }

    pub fn release(&mut self, var: Variable) {
        let locals = self.locals.get_mut(&var.1).unwrap();
        locals.release(var.0);
    }

    pub fn borrow_param(&mut self, idx: LocalIndex) -> Variable {
        let tp = self.input[idx as usize];
        Variable(idx, tp)
    }

    pub fn locals(&self) -> Vec<SType> {
        self.list.to_vec()
    }
}

#[derive(Default, Debug)]
pub struct Locals {
    free: Vec<LocalIndex>,
    borrowed: Vec<LocalIndex>,
}

impl Locals {
    pub fn borrow(&mut self) -> Option<LocalIndex> {
        if let Some(free) = self.free.pop() {
            self.borrowed.push(free);
            Some(free)
        } else {
            None
        }
    }

    pub fn borrow_with_id(&mut self, id: LocalIndex) {
        let local = self
            .free
            .iter()
            .enumerate()
            .find(|(_, i)| **i == id)
            .map(|i| i.0);
        if let Some(idx) = local {
            self.free.remove(idx);
            self.borrowed.push(id);
        }
    }

    pub fn new_borrowed(&mut self, id: LocalIndex) {
        self.borrowed.push(id);
    }

    pub fn release(&mut self, id: LocalIndex) -> bool {
        let borrowed_idx = self
            .borrowed
            .iter()
            .enumerate()
            .find(|(_, b)| **b == id)
            .map(|(i, _)| i);

        if let Some(borrowed_idx) = borrowed_idx {
            self.borrowed.remove(borrowed_idx);
            self.free.push(id);
            true
        } else {
            false
        }
    }

    pub fn contains(&self, idx: LocalIndex) -> bool {
        self.free.contains(&idx) || self.borrowed.contains(&idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Variable(LocalIndex, SType);

impl Variable {
    pub fn none() -> Variable {
        Variable(0, SType::Signer)
    }

    pub fn ty(&self) -> SType {
        self.1
    }

    pub fn is_num(&self) -> bool {
        matches!(self.1, SType::Num)
    }

    pub fn index(&self) -> LocalIndex {
        self.0
    }

    pub fn expr(&self) -> TypedExpr {
        Expression::MoveVar(*self).ty(self.1)
    }

    pub fn assign(&self, expr: Loc<TypedExpr>) -> Statement {
        Statement::Assign(*self, expr)
    }
}
