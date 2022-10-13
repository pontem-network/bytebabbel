use crate::bytecode::hir::ir::{Expr, VarId, _Expr};
use crate::bytecode::loc::Loc;
use anyhow::{bail, ensure, Error};

use crate::bytecode::mir::ir::expression::{Expression, TypedExpr};
use crate::bytecode::mir::ir::types::{LocalIndex, SType};
use crate::bytecode::mir::translation::variables::Variable;
use crate::MirTranslator;

impl<'a> MirTranslator<'a> {
    pub fn translate_expr(&mut self, expr: Expr) -> Result<Loc<TypedExpr>, Error> {
        let loc = expr.wrap(());
        let res = match expr.inner() {
            _Expr::Val(val) => val.into(),
            _Expr::Var(var) => {
                let is_tmp = var.is_tmp();
                let var = self.get_var(var)?;
                if is_tmp {
                    self.vars.release(var);
                }
                Expression::MoveVar(var).ty(var.ty())
            }
            _Expr::MLoad(offset) => {
                let offset = self.translate_expr(*offset)?;
                let offset = self.cast_expr(offset, SType::Num)?;
                Expression::MLoad {
                    memory: self.mem_var,
                    offset,
                }
                .ty(SType::Num)
            }
            _Expr::SLoad(key) => {
                let key = self.translate_expr(*key)?;
                Expression::SLoad {
                    storage: self.store_var,
                    key: self.cast_expr(key, SType::Num)?,
                }
                .ty(SType::Num)
            }
            _Expr::Signer => {
                let signer = self.vars.borrow_param(self.signer_index);
                let signer = Expression::CopyVar(signer).ty(signer.ty()).loc(loc);
                self.cast_expr(signer, SType::Num)?.inner()
            }
            _Expr::MSize => Expression::MSize {
                memory: self.mem_var,
            }
            .ty(SType::Num),
            _Expr::ArgsSize => self.args_size()?,
            _Expr::Args(offset) => self.args(*offset)?,
            _Expr::UnaryOp(op, arg) => self.translate_unary_op(op, *arg)?,
            _Expr::BinaryOp(op, arg1, arg2) => self.translate_binary_op(op, *arg1, *arg2)?,
            _Expr::TernaryOp(op, arg1, arg2, arg3) => {
                self.translate_ternary_op(op, *arg1, *arg2, *arg3)?
            }
            _Expr::Hash(offset, len) => {
                let offset = self.translate_expr(*offset)?;
                let len = self.translate_expr(*len)?;

                ensure!(offset.ty == SType::Num, "offset must be of type num");
                ensure!(len.ty == SType::Num, "len must be of type num");
                Expression::Hash {
                    mem: self.mem_var,
                    offset,
                    len,
                }
                .ty(SType::Num)
            }
            _Expr::Copy(expr) => {
                if let _Expr::Var(var) = expr.inner() {
                    let var = self.get_var(var)?;
                    Expression::CopyVar(var).ty(var.ty())
                } else {
                    bail!("Only variables can be copied")
                }
            }
            _Expr::Balance(address_num) => {
                let address_num = self.translate_expr(*address_num)?;
                let address = self.cast_expr(address_num, SType::Address)?;
                ensure!(
                    address.ty == SType::Address,
                    "address_var must be of type address"
                );
                Expression::Balance { address: address }.ty(SType::Num)
            }
            _Expr::Gas => Expression::Gas.ty(SType::Num),
            _Expr::GasPrice => Expression::GasPrice.ty(SType::Num),
            _Expr::GasLimit => Expression::GasLimit.ty(SType::Num),
            _Expr::BlockHeight => Expression::BlockHeight.ty(SType::Num),
            _Expr::BlockTimestamp => Expression::BlockTimestamp.ty(SType::Num),
            _Expr::BlockHash => Expression::BlockHash.ty(SType::Address),
        };
        Ok(res.loc(loc))
    }

    fn get_var(&mut self, var: VarId) -> Result<Variable, Error> {
        if var.is_tmp() {
            Ok(*self
                .var_map
                .get(&var)
                .ok_or_else(|| anyhow::anyhow!("variable not found: {:?}", var))?)
        } else {
            Ok(*self
                .stack_map
                .get(&var)
                .ok_or_else(|| anyhow::anyhow!("variable not found: {:?}", var))?)
        }
    }

    fn args_size(&mut self) -> Result<TypedExpr, Error> {
        if self.flags.native_input {
            bail!("args_size is not supported in native input mode");
        } else {
            let args = self.vars.borrow_param(self.args_index);
            ensure!(args.ty() == SType::Bytes, "args must be of type bytes");
            Ok(Expression::BytesLen(args).ty(SType::Num))
        }
    }

    fn args(&mut self, offset: Expr) -> Result<TypedExpr, Error> {
        Ok(if self.flags.native_input {
            let param = self.vars.borrow_param(
                offset
                    .as_val()
                    .ok_or_else(|| {
                        anyhow::anyhow!("args offset must be a constant in native input mode")
                    })?
                    .as_u32() as LocalIndex,
            );
            Expression::MoveVar(param).ty(param.ty())
        } else {
            let data = self.vars.borrow_param(self.args_index);
            let offset = self.translate_expr(offset)?;
            ensure!(offset.ty == SType::Num, "offset must be of type num");
            ensure!(data.ty() == SType::Bytes, "args must be of type bytes");
            Expression::ReadNum { data, offset }.ty(SType::Num)
        })
    }
}
