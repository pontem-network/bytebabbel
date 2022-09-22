use crate::bytecode::hir::ir::var::{Expr, VarId};
use crate::bytecode::mir::ir::expression::{Expression, TypedExpr};
use crate::bytecode::mir::ir::types::SType;
use crate::MirTranslator;
use anyhow::{bail, ensure, Error};

impl<'a> MirTranslator<'a> {
    pub fn translate_expr(&mut self, expr: &Expr) -> Result<TypedExpr, Error> {
        Ok(match expr {
            Expr::Val(val) => val.into(),
            Expr::Var(var) => {
                let var = self.get_var(*var)?;
                Expression::Var(var).ty(var.ty())
            }
            Expr::MLoad(offset) => {
                let offset = self.get_var(*offset)?;
                let offset = self.cast(offset, SType::Num)?;
                Expression::MLoad {
                    memory: self.mem_var,
                    offset,
                }
                .ty(SType::Num)
            }
            Expr::SLoad(key) => {
                let key = self.get_var(*key)?;
                let key = self.cast(key, SType::Num)?;
                Expression::SLoad {
                    storage: self.store_var,
                    offset: key,
                }
                .ty(SType::Num)
            }
            Expr::Signer => {
                let signer = self.variables.borrow_param(self.signer_index);
                let result = self.cast(signer, SType::Num)?;
                Expression::Var(result).ty(SType::Num)
            }
            Expr::MSize => Expression::MSize {
                memory: self.mem_var,
            }
            .ty(SType::Num),
            Expr::ArgsSize => self.args_size()?,
            Expr::Args(offset) => self.args(*offset)?,
            Expr::UnaryOp(op, arg) => self.translate_unary_op(*op, arg)?,
            Expr::BinaryOp(op, arg, arg1) => self.translate_binary_op(*op, arg, arg1)?,
            Expr::TernaryOp(op, arg1, arg2, arg3) => {
                self.translate_ternary_op(*op, arg1, arg2, arg3)?
            }
            Expr::Hash(offset, len) => {
                let offset = self.get_var(*offset)?;
                let len = self.get_var(*len)?;

                ensure!(offset.ty() == SType::Num, "offset must be of type num");
                ensure!(len.ty() == SType::Num, "len must be of type num");
                Expression::Hash {
                    mem: self.mem_var,
                    offset,
                    len,
                }
                .ty(SType::Num)
            }
        })
    }

    fn args_size(&mut self) -> Result<TypedExpr, Error> {
        if self.flags.native_input {
            bail!("args_size is not supported in native input mode");
        } else {
            let args = self.variables.borrow_param(self.args_index);
            ensure!(args.ty() == SType::Bytes, "args must be of type bytes");
            Ok(Expression::BytesLen(args).ty(SType::Num))
        }
    }

    fn args(&mut self, offset: VarId) -> Result<TypedExpr, Error> {
        Ok(if self.flags.native_input {
            let param = self.variables.borrow_param(offset.local_index());
            Expression::Var(param).ty(param.ty())
        } else {
            let data = self.variables.borrow_param(self.args_index);
            let offset = self.get_var(offset)?;
            ensure!(offset.ty() == SType::Num, "offset must be of type num");
            ensure!(data.ty() == SType::Bytes, "args must be of type bytes");
            Expression::ReadNum { data, offset }.ty(SType::Num)
        })
    }
}
