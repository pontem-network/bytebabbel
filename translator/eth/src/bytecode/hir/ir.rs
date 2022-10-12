use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::debug::print_stmt;
use crate::bytecode::hir::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use crate::bytecode::hir::vars::Vars;
use crate::bytecode::loc::Loc;
use crate::Offset;
use anyhow::Error;
use primitive_types::U256;
use std::collections::BTreeMap;
use std::fmt::{Debug, Display, Formatter, Write};

#[derive(Debug, Clone, Default)]
pub struct Hir {
    statement: Vec<Loc<Stmt>>,
    labels: BTreeMap<Label, usize>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Label(Label),
    StoreStack(BTreeMap<VarId, Expr>),
    Assign(VarId, Expr),
    MemStore8 {
        addr: Expr,
        val: Expr,
    },
    MemStore {
        addr: Expr,
        val: Expr,
    },
    SStore {
        key: Expr,
        val: Expr,
    },
    Log {
        offset: Expr,
        len: Expr,
        topics: Vec<Expr>,
    },
    Stop,
    Abort(u8),
    Result {
        offset: Expr,
        len: Expr,
    },
    BrunchTrue(Expr, Label),
    Brunch(Label),
}

pub type Expr = Loc<_Expr>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum _Expr {
    Val(U256),
    Var(VarId),
    MLoad(Box<Expr>),
    SLoad(Box<Expr>),
    Signer,
    MSize,
    ArgsSize,
    Args(Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
    BinaryOp(BinaryOp, Box<Expr>, Box<Expr>),
    TernaryOp(TernaryOp, Box<Expr>, Box<Expr>, Box<Expr>),
    Hash(Box<Expr>, Box<Expr>),
    Copy(Box<Expr>),
}

impl Expr {
    pub fn unvar(&self, ctx: &Context) -> Expr {
        match self.as_ref() {
            _Expr::Var(id) => {
                let expr = ctx.vars.get(id).expect("variable not found").clone();
                expr.unvar(ctx)
            }
            _Expr::Val(_) | _Expr::Signer | _Expr::MSize | _Expr::ArgsSize => self.clone(),
            _Expr::MLoad(expr) => {
                let expr = expr.unvar(ctx);
                self.wrap(_Expr::MLoad(Box::new(expr)))
            }
            _Expr::SLoad(expr) => {
                let expr = expr.unvar(ctx);
                self.wrap(_Expr::SLoad(Box::new(expr)))
            }
            _Expr::Args(expr) => {
                let expr = expr.unvar(ctx);
                self.wrap(_Expr::Args(Box::new(expr)))
            }
            _Expr::UnaryOp(cmd, expr) => {
                let expr = expr.unvar(ctx);
                self.wrap(_Expr::UnaryOp(*cmd, Box::new(expr)))
            }
            _Expr::BinaryOp(cmd, expr1, expr2) => {
                let expr1 = expr1.unvar(ctx);
                let expr2 = expr2.unvar(ctx);
                self.wrap(_Expr::BinaryOp(*cmd, Box::new(expr1), Box::new(expr2)))
            }
            _Expr::TernaryOp(cmd, expr1, expr2, expr3) => {
                let expr1 = expr1.unvar(ctx);
                let expr2 = expr2.unvar(ctx);
                let expr3 = expr3.unvar(ctx);
                self.wrap(_Expr::TernaryOp(
                    *cmd,
                    Box::new(expr1),
                    Box::new(expr2),
                    Box::new(expr3),
                ))
            }
            _Expr::Hash(expr, expr1) => {
                let expr = expr.unvar(ctx);
                let expr1 = expr1.unvar(ctx);
                self.wrap(_Expr::Hash(Box::new(expr), Box::new(expr1)))
            }
            _Expr::Copy(expr) => {
                let expr = expr.unvar(ctx);
                self.wrap(_Expr::Copy(Box::new(expr)))
            }
        }
    }
}

impl _Expr {
    pub fn resolve(&self, ir: &Hir, ctx: &Context) -> Option<U256> {
        match self {
            _Expr::Val(val) => Some(*val),
            _Expr::Var(var) => {
                let expr = ctx.vars.get(var)?;
                expr.resolve(ir, ctx)
            }
            _Expr::MLoad(_) => None,
            _Expr::SLoad(_) => None,
            _Expr::Signer => None,
            _Expr::MSize => None,
            _Expr::ArgsSize => None,
            _Expr::Args(_) => None,
            _Expr::UnaryOp(cnd, arg) => {
                let arg = arg.resolve(ir, ctx)?;
                Some(cnd.calc(arg))
            }
            _Expr::BinaryOp(cnd, arg1, arg2) => {
                let arg1 = arg1.resolve(ir, ctx)?;
                let arg2 = arg2.resolve(ir, ctx)?;
                Some(cnd.calc(arg1, arg2))
            }
            _Expr::TernaryOp(cnd, arg1, arg2, arg3) => {
                let arg1 = arg1.resolve(ir, ctx)?;
                let arg2 = arg2.resolve(ir, ctx)?;
                let arg3 = arg3.resolve(ir, ctx)?;
                Some(cnd.calc(arg1, arg2, arg3))
            }
            _Expr::Hash(_, _) => None,
            _Expr::Copy(expr) => expr.resolve(ir, ctx),
        }
    }

    pub fn is_var(&self) -> bool {
        matches!(self, _Expr::Var(_))
    }

    pub fn as_val(&self) -> Option<U256> {
        match self {
            _Expr::Val(val) => Some(*val),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy, Ord, PartialOrd)]
pub struct Label {
    to: Offset,
    from: Option<Offset>,
}

impl Label {
    pub fn new(to: Offset) -> Self {
        Self { to, from: None }
    }

    pub fn from(self, from: Offset) -> Self {
        Self {
            from: Some(from),
            ..self
        }
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(from) = self.from {
            write!(f, "'{}_{}", from, self.to)
        } else {
            write!(f, "'{}", self.to)
        }
    }
}

impl Hir {
    pub fn assign(&mut self, expr: Expr, vars: &mut Vars) -> VarId {
        let var = vars.gen_tmp();
        self.statement
            .push(expr.wrap(Stmt::Assign(var, expr.clone())));
        vars.set(var, expr);
        var
    }

    pub fn abort(&mut self, loc: &Loc<()>, code: u8) {
        self.statement.push(loc.wrap(Stmt::Abort(code)));
    }

    pub fn result(&mut self, loc: &Loc<()>, offset: Expr, len: Expr) {
        self.statement.push(loc.wrap(Stmt::Result { offset, len }));
    }

    pub fn stop(&mut self, loc: &Loc<()>) {
        self.statement.push(loc.wrap(Stmt::Stop));
    }

    pub fn return_(&mut self, loc: &Loc<()>, offset: Expr, len: Expr) {
        self.statement.push(loc.wrap(Stmt::Result { offset, len }));
    }

    pub fn mstore(&mut self, loc: &Loc<()>, addr: Expr, var: Expr) {
        self.statement
            .push(loc.wrap(Stmt::MemStore { addr, val: var }));
    }

    pub fn mstore8(&mut self, loc: &Loc<()>, addr: Expr, var: Expr) {
        self.statement
            .push(loc.wrap(Stmt::MemStore8 { addr, val: var }));
    }

    pub fn save_stack(&mut self, loc: &Loc<()>, context: BTreeMap<VarId, Expr>) {
        self.statement.push(loc.wrap(Stmt::StoreStack(context)));
    }

    pub fn sstore(&mut self, loc: &Loc<()>, addr: Expr, var: Expr) {
        self.statement.push(loc.wrap(Stmt::SStore {
            key: addr,
            val: var,
        }));
    }

    pub fn true_brunch(&mut self, loc: &Loc<()>, cnd: Expr, label: Label) {
        self.statement.push(loc.wrap(Stmt::BrunchTrue(cnd, label)));
    }

    pub fn log(&mut self, loc: &Loc<()>, offset: Expr, len: Expr, topics: Vec<Expr>) {
        self.statement.push(loc.wrap(Stmt::Log {
            offset,
            len,
            topics,
        }));
    }

    pub fn label(&mut self, loc: &Loc<()>, label: Label) {
        self.statement.push(loc.wrap(Stmt::Label(label)));
        self.labels.insert(label, self.statement.len() - 1);
    }

    pub fn has_label(&self, label: Label) -> bool {
        self.labels.contains_key(&label)
    }

    pub fn goto(&mut self, loc: &Loc<()>, label: Label) {
        self.statement.push(loc.wrap(Stmt::Brunch(label)));
    }

    pub fn statements(&self) -> &[Loc<Stmt>] {
        &self.statement
    }

    pub fn inner(self) -> Vec<Loc<Stmt>> {
        self.statement
    }

    pub fn print<B: Write>(&self, buf: &mut B) -> Result<(), Error> {
        for stmt in &self.statement {
            print_stmt(buf, stmt)?;
        }
        Ok(())
    }
}

impl From<U256> for _Expr {
    fn from(val: U256) -> Self {
        _Expr::Val(val)
    }
}

impl From<u128> for _Expr {
    fn from(val: u128) -> Self {
        _Expr::Val(U256::from(val))
    }
}

impl From<VarId> for _Expr {
    fn from(id: VarId) -> Self {
        _Expr::Var(id)
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct VarId(u32, bool);

impl VarId {
    pub fn new_var(idx: u32) -> Self {
        VarId(idx, false)
    }

    pub fn new_tmp(idx: u32) -> Self {
        VarId(idx, true)
    }

    pub fn is_tmp(&self) -> bool {
        self.1
    }

    pub fn id(&self) -> u32 {
        self.0
    }
}

impl Debug for VarId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for VarId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.1 {
            write!(f, "tmp{}", self.0)
        } else {
            write!(f, "var{}", self.0)
        }
    }
}
