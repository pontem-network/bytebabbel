use crate::bytecode::hir2::const_pool::ConstPool;
use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::ir::expression::Expr;
use crate::bytecode::hir2::ir::statement::Statement;
use crate::bytecode::hir2::ir::Hir2;
use crate::bytecode::hir2::vars::VarId;
use std::collections::HashMap;
use std::rc::Rc;

pub fn constant_fold(hir: Hir2, ctx: &mut Context) -> Hir2 {
    let mut vars = Vars::new(ctx.const_pool());
    map_ir(hir, &mut vars)
}

fn map_ir(hir: Hir2, vars: &mut Vars) -> Hir2 {
    Hir2::from(map_sts(hir.statements(), vars))
}

fn map_sts(statements: &[Statement], vars: &mut Vars) -> Vec<Statement> {
    let mut new_statements = Vec::new();
    for statement in statements {
        if let Some(new_statement) = map_st(statement, vars) {
            new_statements.push(new_statement);
        }
    }
    new_statements
}

fn map_st(statement: &Statement, vars: &mut Vars) -> Option<Statement> {
    Some(match statement {
        Statement::Assign { var, expr } => {
            if let Some(cnst) = vars.pool.get_const(&var) {
                if cnst.users < 2 {
                    return None;
                }
            }
            Statement::Assign {
                var: vars.make_mapping(*var),
                expr: map_expr(&expr, vars),
            }
        }
        Statement::MemStore8 { addr, var } => Statement::MemStore8 {
            addr: map_expr(addr, vars),
            var: map_expr(var, vars),
        },
        Statement::MemStore { addr, var } => Statement::MemStore {
            addr: map_expr(addr, vars),
            var: map_expr(var, vars),
        },
        Statement::SStore { addr, var } => Statement::SStore {
            addr: map_expr(addr, vars),
            var: map_expr(var, vars),
        },
        Statement::Log {
            offset,
            len,
            topics,
        } => Statement::Log {
            offset: map_expr(offset, vars),
            len: map_expr(len, vars),
            topics: topics.into_iter().map(|t| map_expr(t, vars)).collect(),
        },
        Statement::If {
            condition,
            true_branch,
            false_branch,
        } => Statement::If {
            condition: map_expr(condition, vars),
            true_branch: map_sts(true_branch, vars),
            false_branch: map_sts(false_branch, vars),
        },
        Statement::Loop {
            id,
            condition_block,
            condition,
            is_true_br_loop,
            loop_br,
        } => Statement::Loop {
            id: *id,
            condition_block: map_sts(condition_block, vars),
            condition: map_expr(condition, vars),
            is_true_br_loop: *is_true_br_loop,
            loop_br: map_sts(loop_br, vars),
        },
        Statement::Continue { loop_id, context } => Statement::Continue {
            loop_id: *loop_id,
            context: map_sts(context, vars),
        },
        Statement::Stop => Statement::Stop,
        Statement::Abort(code) => Statement::Abort(*code),
        Statement::Result { offset, len } => Statement::Result {
            offset: map_expr(offset, vars),
            len: map_expr(len, vars),
        },
        Statement::Move(expr) => Statement::Move(map_expr(expr, vars)),
    })
}

fn map_expr(expr: &Rc<Expr>, vars: &mut Vars) -> Rc<Expr> {
    Rc::new(match expr.as_ref() {
        Expr::Val(_) | Expr::Signer | Expr::MSize | Expr::ArgsSize => return expr.clone(),
        Expr::Var(var) => {
            if let Some(cnst) = vars.pool.get_const(&var) {
                if cnst.users < 2 {
                    return Rc::new(Expr::Val(cnst.val));
                }
            }
            Expr::Var(vars.map_var(*var))
        }
        Expr::MLoad { mem_offset } => Expr::MLoad {
            mem_offset: map_expr(mem_offset, vars),
        },
        Expr::SLoad { key } => Expr::SLoad {
            key: map_expr(key, vars),
        },
        Expr::Args { args_offset } => Expr::Args {
            args_offset: map_expr(args_offset, vars),
        },
        Expr::UnaryOp(cmd, op) => Expr::UnaryOp(*cmd, map_expr(op, vars)),
        Expr::BinaryOp(cmd, op1, op2) => {
            Expr::BinaryOp(*cmd, map_expr(op1, vars), map_expr(op2, vars))
        }
        Expr::TernaryOp(cmd, op1, op2, op3) => Expr::TernaryOp(
            *cmd,
            map_expr(op1, vars),
            map_expr(op2, vars),
            map_expr(op3, vars),
        ),
        Expr::Hash {
            mem_offset,
            mem_len,
        } => Expr::Hash {
            mem_offset: map_expr(mem_offset, vars),
            mem_len: map_expr(mem_len, vars),
        },
    })
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

    fn map_var(&self, var: VarId) -> VarId {
        self.mapping.get(&var).copied().expect("var not found")
    }

    fn make_mapping(&mut self, var: VarId) -> VarId {
        *self.mapping.entry(var).or_insert_with(|| {
            let new_var = VarId::from(self.seq);
            self.seq += 1;
            new_var
        })
    }
}
