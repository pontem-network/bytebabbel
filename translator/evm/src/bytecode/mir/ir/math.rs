#[derive(Debug)]
pub enum Operation {
    /// Add the 2 u64 at the top of the stack and pushes the result on the stack.
    /// The operation aborts the transaction in case of overflow.
    Add,
    /// Subtract the 2 u64 at the top of the stack and pushes the result on the stack.
    /// The operation aborts the transaction in case of underflow.
    Sub,
    /// Multiply the 2 u64 at the top of the stack and pushes the result on the stack.
    /// The operation aborts the transaction in case of overflow.
    Mul,
    /// Perform a modulo operation on the 2 u64 at the top of the stack and pushes the
    /// result on the stack.
    Mod,
    /// Divide the 2 u64 at the top of the stack and pushes the result on the stack.
    /// The operation aborts the transaction in case of "divide by 0".
    Div,
    /// Bitwise OR the 2 u64 at the top of the stack and pushes the result on the stack.
    BitOr,
    /// Bitwise AND the 2 u64 at the top of the stack and pushes the result on the stack.
    BitAnd,
    /// Bitwise XOR the 2 u64 at the top of the stack and pushes the result on the stack.
    Xor,
    /// Logical OR the 2 bool at the top of the stack and pushes the result on the stack.
    Or,
    /// Logical AND the 2 bool at the top of the stack and pushes the result on the stack.
    And,
    /// Logical NOT the bool at the top of the stack and pushes the result on the stack.
    Not,
    /// Compare for equality the 2 value at the top of the stack and pushes the
    /// result on the stack.
    /// The values on the stack must have `Drop` as they will be consumed and destroyed.
    Eq,
    /// Compare for inequality the 2 value at the top of the stack and pushes the
    /// result on the stack.
    /// The values on the stack must have `Drop` as they will be consumed and destroyed.
    Neq,
    /// Perform a "less than" operation of the 2 u64 at the top of the stack and pushes the
    /// result on the stack.
    Lt,
    /// Perform a "greater than" operation of the 2 u64 at the top of the stack and pushes the
    /// result on the stack.
    Gt,
    /// Perform a "less than or equal" operation of the 2 u64 at the top of the stack and pushes
    /// the result on the stack.
    Le,
    /// Perform a "greater than or equal" than operation of the 2 u64 at the top of the stack
    /// and pushes the result on the stack.
    Ge,
    /// Shift the (second top value) left (top value) bits and pushes the result on the stack.
    Shl,
    /// Shift the (second top value) right (top value) bits and pushes the result on the stack.
    Shr,
}
