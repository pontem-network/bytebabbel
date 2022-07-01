use crate::evm::instruction::Offset;
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

    pub fn wrap<R>(&self, inner: R) -> Loc<R> {
        Loc::new(self.start, self.end, inner)
    }

    pub fn inner(self) -> C {
        self.inner
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
