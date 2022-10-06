use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use anyhow::{anyhow, Error};

use crate::bytecode::mir::ir::expression::{Expression, TypedExpr};
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::{LocalIndex, SType};

#[derive(Debug)]
pub struct Variables {
    inner: Rc<RefCell<Inner>>,
}

#[derive(Debug)]
pub struct Inner {
    locals: HashMap<SType, Locals>,
    seq: LocalIndex,
    list: Vec<SType>,
    input: Vec<SType>,
}

impl Variables {
    pub fn new(params: Vec<SType>) -> Variables {
        Variables {
            inner: Rc::new(RefCell::new(Inner {
                locals: HashMap::new(),
                seq: params.len() as LocalIndex,
                list: vec![],
                input: params,
            })),
        }
    }

    pub fn borrow(&mut self, tp: SType) -> Variable {
        let mut vars = self.inner.borrow_mut();
        let idx = vars.seq;
        let locals = vars.locals.entry(tp).or_default();
        if let Some(idx) = locals.borrow() {
            Variable(idx, tp)
        } else {
            locals.new_borrowed(idx);
            vars.seq += 1;
            vars.list.push(tp);
            Variable(idx, tp)
        }
    }

    pub fn release(&mut self, var: Variable) {
        let mut vars = self.inner.borrow_mut();
        let locals = vars.locals.get_mut(&var.1).unwrap();
        locals.release(var.0);
    }

    pub fn borrow_param(&mut self, idx: LocalIndex) -> Variable {
        let vars = self.inner.borrow();
        let tp = vars.input[idx as usize];
        Variable(idx, tp)
    }

    pub fn locals(&self) -> Vec<SType> {
        self.inner.borrow().list.to_vec()
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

    pub fn borrow_with_id(&mut self, id: LocalIndex) -> Option<LocalIndex> {
        let local = self
            .free
            .iter()
            .enumerate()
            .find(|(_, i)| **i == id)
            .map(|i| i.0);
        if let Some(idx) = local {
            self.free.remove(idx);
            self.borrowed.push(id);
            Some(id)
        } else {
            None
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
        Expression::Var(*self).ty(self.1)
    }

    pub fn assign(&self, expr: TypedExpr) -> Statement {
        Statement::Assign(*self, expr)
    }
}
