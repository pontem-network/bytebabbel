use crate::bytecode::instruction::{Instruction, Offset};
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};

pub struct Loc<C> {
    pub start: Offset,
    pub end: Offset,
    inner: C,
}

impl<C> Loc<C> {
    pub fn new(start: Offset, end: Offset, inner: C) -> Loc<C> {
        Loc { start, end, inner }
    }

    pub fn map<F: FnOnce(C) -> R, R>(self, f: F) -> Loc<R> {
        Loc::new(self.start, self.end, f(self.inner))
    }

    pub fn contains(&self, offset: Offset) -> bool {
        self.start <= offset && self.end >= offset
    }

    pub fn wrap<R>(&self, inner: R) -> Loc<R> {
        Loc::new(self.start, self.end, inner)
    }

    pub fn inner(self) -> C {
        self.inner
    }
}

pub trait Move {
    fn move_forward(&mut self, offset: usize);
    fn move_back(&mut self, offset: usize);
}

impl<C: Move> Move for Loc<C> {
    fn move_forward(&mut self, offset: usize) {
        self.start += offset;
        self.end += offset;
        self.inner.move_forward(offset);
    }

    fn move_back(&mut self, offset: usize) {
        self.start -= offset;
        self.end -= offset;
        self.inner.move_back(offset);
    }
}

impl Move for Loc<Vec<Instruction>> {
    fn move_forward(&mut self, offset: usize) {
        self.start += offset;
        self.end += offset;

        for inst in &mut self.inner {
            inst.0 += offset;
        }
    }

    fn move_back(&mut self, offset: usize) {
        self.start -= offset;
        self.end -= offset;
        for inst in &mut self.inner {
            inst.0 -= offset;
        }
    }
}

impl<C> Deref for Loc<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<C> DerefMut for Loc<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<C: Debug> Debug for Loc<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\nLoc: [{}; {}]\n{:?}", self.start, self.end, self.inner)
    }
}

impl<C: Display> Display for Loc<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\nBlock: [{}; {}]\n{}", self.start, self.end, self.inner)
    }
}

impl<C: Clone> Clone for Loc<C> {
    fn clone(&self) -> Self {
        Self {
            start: self.start,
            end: self.end,
            inner: self.inner.clone(),
        }
    }
}
