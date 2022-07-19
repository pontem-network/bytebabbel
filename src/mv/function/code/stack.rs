use move_binary_format::file_format::SignatureToken;

#[derive(Default)]
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
}
