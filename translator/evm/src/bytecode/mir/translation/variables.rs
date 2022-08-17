use crate::bytecode::mir::ir::expression::Expression;
use crate::bytecode::mir::ir::types::{LocalIndex, SType};
use anyhow::{anyhow, Error};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub struct Variables {
    inner: Rc<RefCell<Inner>>,
}

pub struct Inner {
    locals: HashMap<SType, Locals>,
    seq: LocalIndex,
    scopes: Vec<HashSet<Variable>>,
    list: Vec<SType>,
}

impl Variables {
    pub fn new(params_count: LocalIndex) -> Variables {
        Variables {
            inner: Rc::new(RefCell::new(Inner {
                locals: HashMap::new(),
                seq: params_count,
                scopes: Vec::new(),
                list: vec![],
            })),
        }
    }

    pub fn borrow_global(&mut self, tp: SType) -> Variable {
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

    pub fn borrow(&mut self, tp: SType) -> Variable {
        let mut vars = self.inner.borrow_mut();
        let idx = vars.seq;
        let locals = vars.locals.entry(tp).or_default();
        let var = if let Some(idx) = locals.borrow() {
            Variable(idx, tp)
        } else {
            locals.new_borrowed(idx);
            vars.seq += 1;
            vars.list.push(tp);
            Variable(idx, tp)
        };
        let current_scope = vars.scopes.len() - 1;
        if let Some(scope) = vars.scopes.get_mut(current_scope) {
            scope.insert(var);
        } else {
            panic!("no scope");
        }
        var
    }

    pub fn borrow_with_id(&mut self, id: LocalIndex, tp: SType) -> Result<Variable, Error> {
        let mut vars = self.inner.borrow_mut();
        let locals = vars.locals.entry(tp).or_default();
        let idx = locals
            .borrow_with_id(id)
            .ok_or_else(|| anyhow!("{} is not a valid local index", id))?;
        Ok(Variable(idx, tp))
    }

    pub fn create_scope(&mut self) -> Scope {
        {
            let mut vars = self.inner.borrow_mut();
            vars.scopes.push(HashSet::new());
        };
        Scope {
            vars: Variables {
                inner: self.inner.clone(),
            },
        }
    }

    pub fn locals(&self) -> Vec<SType> {
        self.inner.borrow().list.to_vec()
    }
}

#[derive(Default)]
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

pub struct Scope {
    vars: Variables,
}

impl Drop for Scope {
    fn drop(&mut self) {
        let mut vars = self.vars.inner.borrow_mut();
        if let Some(scope) = vars.scopes.pop() {
            for var in scope {
                if let Some(locals) = vars.locals.get_mut(&var.s_type()) {
                    locals.release(var.0);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Variable(LocalIndex, SType);

impl Variable {
    pub fn s_type(&self) -> SType {
        self.1
    }

    pub fn index(&self) -> LocalIndex {
        self.0
    }

    pub fn expr(&self) -> Expression {
        Expression::Var(*self)
    }
}
