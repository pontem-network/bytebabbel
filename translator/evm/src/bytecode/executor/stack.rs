use crate::bytecode::block::BlockId;
use crate::bytecode::executor::ops::{BinaryOp, UnaryOp};
use crate::bytecode::executor::types::U256;
use std::cell::Cell;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

pub const FRAME_SIZE: usize = 32;

#[derive(Default, Clone)]
pub struct Stack {
    pub stack: Vec<StackFrame>,
}

impl Stack {
    pub fn clean(&mut self) {
        self.stack.clear();
    }

    pub fn pop(&mut self, count: usize) -> Vec<StackFrame> {
        let mut res = Vec::with_capacity(count);
        for _ in 0..count {
            if let Some(item) = self.stack.pop() {
                res.push(item);
            }
        }
        res
    }

    pub fn push(&mut self, to_push: Vec<StackFrame>) {
        self.stack.extend(to_push.into_iter().rev());
    }
}

#[derive(Clone)]
pub struct StackFrame {
    cell: Rc<Frame>,
    used: Used,
}

impl StackFrame {
    pub fn new(cell: Frame) -> StackFrame {
        StackFrame {
            cell: Rc::new(cell),
            used: Default::default(),
        }
    }

    pub fn mark_as_used(&self) {
        self.used.mark_as_used();
    }

    pub fn is_used(&self) -> bool {
        self.used.0.get()
    }

    pub fn get_used_flag(&self) -> Used {
        self.used.clone()
    }

    pub fn set_used_flag(&mut self, used: Used) {
        self.used = used;
    }

    pub fn as_usize(&self) -> Option<usize> {
        self.as_u256().map(|val| val.as_usize())
    }

    pub fn as_u256(&self) -> Option<U256> {
        if let Frame::Val(val) = self.cell.as_ref() {
            Some(*val)
        } else if let Frame::Bool(val) = self.cell.as_ref() {
            Some(if *val { U256::from(1) } else { U256::from(0) })
        } else {
            None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        Some(self.as_u256()? != U256::from(0))
    }

    pub fn as_block_id(&self) -> Option<BlockId> {
        self.as_usize().map(|i| i.into())
    }

    pub fn frame(&self) -> Rc<Frame> {
        self.cell.clone()
    }
}

#[derive(Hash, Eq, PartialEq)]
pub enum Frame {
    Val(U256),
    Param(u16),
    Bool(bool),
    SelfAddress,
    Mem(Box<StackFrame>, Box<StackFrame>),
    Calc(UnaryOp, StackFrame),
    Calc2(BinaryOp, StackFrame, StackFrame),
    Abort(u64),
}

impl Debug for Frame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Frame::Val(val) => {
                write!(f, "{:#06x}", val)
            }
            Frame::Param(idx) => write!(f, "param: {idx})"),
            Frame::SelfAddress => write!(f, "addr"),
            Frame::Calc2(op, a, b) => write!(f, "{op:?}({a:?}, {b:?}))"),
            Frame::Mem(rf, mem) => write!(f, "mem {:?} => {:?}", rf, mem),
            Frame::Bool(val) => write!(f, "{val}"),
            Frame::Abort(code) => write!(f, "abort {code}"),
            Frame::Calc(op, mem) => write!(f, "{op:?}({mem:?})"),
        }
    }
}

impl Debug for StackFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.cell)
    }
}

#[derive(Default, Clone)]
pub struct Used(Rc<Cell<bool>>);

impl Used {
    pub fn mark_as_used(&self) {
        self.0.set(true);
    }
}

impl Debug for Used {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.0.get() {
            write!(f, "used")
        } else {
            write!(f, "unused")
        }
    }
}

impl Display for Used {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Hash for StackFrame {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.cell.as_ref().hash(state)
    }
}

impl PartialEq<Self> for StackFrame {
    fn eq(&self, other: &Self) -> bool {
        self.cell.eq(&other.cell)
    }
}

impl Eq for StackFrame {}
