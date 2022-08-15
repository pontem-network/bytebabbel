use crate::bytecode::mir::ir::types::{LocalIndex, SType};
use anyhow::{anyhow, Error};
use std::collections::HashMap;

pub struct Variables {
    locals: HashMap<SType, Locals>,
    seq: LocalIndex,
}

impl Variables {
    pub fn new(params_count: LocalIndex) -> Variables {
        Variables {
            locals: HashMap::new(),
            seq: params_count,
        }
    }

    pub fn borrow(&mut self, tp: SType) -> LocalIndex {
        let locals = self.locals.entry(tp).or_default();
        if let Some(idx) = locals.borrow() {
            idx
        } else {
            let idx = self.seq;
            self.seq += 1;
            locals.new_borrowed(idx);
            idx
        }
    }

    pub fn borrow_with_id(&mut self, id: LocalIndex, tp: SType) -> Result<LocalIndex, Error> {
        let locals = self.locals.entry(tp).or_default();
        locals
            .borrow_with_id(id)
            .ok_or_else(|| anyhow!("{} is not a valid local index", id))
    }

    pub fn check_type(&self, tp: SType, local: LocalIndex) -> Result<(), Error> {
        if let Some(locals) = self.locals.get(&tp) {
            if locals.contains(local) {
                Ok(())
            } else {
                Err(anyhow!("local {} is not of type {:?}", local, tp))
            }
        } else {
            Err(anyhow!("local {} is not of type {:?}", local, tp))
        }
    }

    pub fn release(&mut self, idx: LocalIndex) -> Option<SType> {
        for (tp, locals) in self.locals.iter_mut() {
            if locals.release(idx) {
                return Some(*tp);
            }
        }
        None
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
