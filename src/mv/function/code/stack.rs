use move_binary_format::file_format::SignatureToken;
use std::fmt::{Display, Formatter};

#[derive(Default, Debug, Clone)]
pub struct Stack {
    inner: Vec<SignatureToken>,
}

impl Stack {
    pub fn push(&mut self, tp: SignatureToken) {
        self.inner.push(tp);
    }

    pub fn pop(&mut self) -> SignatureToken {
        self.inner.pop().expect("Expected positive stack size")
    }

    pub fn pop2(&mut self) -> [SignatureToken; 2] {
        [self.pop(), self.pop()]
    }

    pub fn into_inner(self) -> Vec<SignatureToken> {
        self.inner
    }
}

impl Display for Stack {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}
