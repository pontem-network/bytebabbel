use crate::bytecode::hir2::const_pool::ConstPool;
use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::ir::expression::Expr;
use crate::bytecode::hir2::ir::statement::Statement;
use crate::bytecode::hir2::ir::Hir2;
use crate::bytecode::hir2::vars::VarId;
use anyhow::Error;
use std::collections::HashMap;
use std::rc::Rc;

pub fn const_fold(hir: Hir2, ctx: &mut Context) -> Result<Hir2, Error> {
    let mut vars = Vars::new(ctx.const_pool());
    map_ir(hir, &mut vars)
}

fn map_ir(hir: Hir2, vars: &mut Vars) -> Result<Hir2, Error> {
    let mut new_hir = Hir2::default();
    for statement in hir.inner() {
        if let Some(new_statement) = map_statement(statement, vars)? {
            new_hir.add_statement(new_statement);
        }
    }
    Ok(new_hir)
}

fn map_statement(statement: Statement, vars: &mut Vars) -> Result<Option<Statement>, Error> {
    match statement {
        Statement::Assign { var, expr } => {
            vars.pool.get_const(var);
            let new_expr = map_expression(expr, vars)?;

            Ok(Some(Statement::Assign {
                var,
                expr: new_expr,
            }))
        }
    }
}

fn map_expression(expr: Rc<Expr>, vars: &mut Vars) -> Result<Rc<Expr>, Error> {
    match expr {}
}

struct Vars<'a> {
    mapping: HashMap<VarId, VarId>,
    pool: &'a mut ConstPool,
    seq: u64,
}

impl<'a> Vars<'a> {
    fn new(pool: &'a mut ConstPool) -> Self {
        Self {
            mapping: Default::default(),
            pool,
            seq: 1,
        }
    }
}
