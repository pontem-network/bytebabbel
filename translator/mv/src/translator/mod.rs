use crate::mv_ir::func::Func;
use crate::mv_ir::Module;
use crate::translator::signature::{map_signature, SignatureWriter};
use crate::translator::writer::Writer;
use anyhow::{anyhow, Error};
use evm::bytecode::block::BlockId;
use evm::bytecode::mir::ir::expression::{Expression, StackOp};
use evm::bytecode::mir::ir::math::Operation;
use evm::bytecode::mir::ir::statement::Statement;
use evm::bytecode::mir::ir::types::SType;
use evm::bytecode::mir::ir::Mir;
use evm::function::FunDef;
use evm::program::Program;
use move_binary_format::file_format::{Bytecode, SignatureIndex, SignatureToken, Visibility};
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;

pub mod bytecode;
pub mod signature;
pub mod writer;

#[derive(Default)]
pub struct MvIrTranslator {
    sign_writer: SignatureWriter,
    code: Writer,
}

impl MvIrTranslator {
    pub fn translate(mut self, address: AccountAddress, program: Program) -> Result<Module, Error> {
        let name = Identifier::new(program.name())?;

        let funcs = program
            .public_functions()
            .into_iter()
            .map(|def| self.translate_func(def, &program))
            .collect::<Result<_, _>>()?;

        Ok(Module {
            address,
            name,
            funcs,
            signatures: self.sign_writer.freeze(),
        })
    }

    fn translate_func(&mut self, def: FunDef, program: &Program) -> Result<Func, Error> {
        let name = Identifier::new(def.abi.name().as_deref().unwrap_or("anonymous"))?;
        let visibility = Visibility::Public;
        let input = self
            .sign_writer
            .make_signature(map_signature(def.abi.inputs().unwrap().as_slice()));

        let output = self
            .sign_writer
            .make_signature(map_signature(def.abi.outputs().unwrap().as_slice()));

        let mir = program
            .function_mir(def.hash)
            .ok_or_else(|| anyhow!("Function {} not found", def.hash))?;

        let locals = self.map_locals(mir);
        self.code.reset();
        self.translate_statements(mir.statements())?;
        let code = self.code.freeze()?;

        Ok(Func {
            name,
            visibility,
            input,
            output,
            locals,
            code,
        })
    }

    fn map_locals(&mut self, mir: &Mir) -> SignatureIndex {
        let types = mir
            .locals()
            .iter()
            .map(|tp| match tp {
                SType::U128 => SignatureToken::U128,
                SType::Bool => SignatureToken::Bool,
            })
            .collect();
        self.sign_writer.make_signature(types)
    }

    fn translate_statements(&mut self, statements: &[Statement]) -> Result<(), Error> {
        for st in statements {
            self.translate_statement(st)?;
        }
        Ok(())
    }

    fn translate_statement(&mut self, st: &Statement) -> Result<(), Error> {
        match st {
            Statement::CreateVar(var, exp) => {
                self.translate_expr(exp)?;
                self.code.set_var(var.index());
            }
            Statement::IF {
                cnd,
                true_br,
                false_br,
            } => {
                self.translate_if(cnd, true_br, false_br)?;
            }
            Statement::Loop {
                id,
                cnd_calc,
                cnd,
                body,
            } => {
                self.translate_loop(*id, cnd_calc, cnd, body)?;
            }
            Statement::Abort(code) => {
                self.code.abort(*code);
            }
            Statement::Result(vars) => {
                for var in vars {
                    self.code.ld_var(var.index());
                }
                self.code.write(Bytecode::Ret);
            }
            Statement::Continue(id) => {
                self.code.mark_jmp_to_label(*id);
            }
        }
        Ok(())
    }

    fn translate_expr(&mut self, exp: &Expression) -> Result<(), Error> {
        match exp {
            Expression::Const(val) => {
                self.code.push_val(val);
            }
            Expression::Var(var) => {
                self.code.ld_var(var.index());
            }
            Expression::Param(idx, _) => {
                self.code.ld_var(*idx);
            }
            Expression::Operation(cmd, op, op1) => {
                if *cmd == Operation::Not {
                    self.code.ld_var(op.index());
                } else {
                    self.code.ld_var(op.index());
                    self.code.ld_var(op1.index());
                }
                self.code.op(*cmd);
            }
            Expression::StackOps(ops) => {
                for op in &ops.vec {
                    match op {
                        StackOp::PushConst(val) => {
                            self.code.push_val(val);
                        }
                        StackOp::PushVar(var) => {
                            self.code.ld_var(var.index());
                        }
                        StackOp::BinaryOp(op) => {
                            self.code.op(*op);
                        }
                        StackOp::Not => {
                            self.code.op(Operation::Not);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn translate_if(
        &mut self,
        cnd: &Expression,
        true_br: &[Statement],
        false_br: &[Statement],
    ) -> Result<(), Error> {
        self.translate_expr(cnd)?;
        let before = self.code.swap(Writer::default());
        self.translate_statements(true_br)?;
        let true_br = self.code.swap(Writer::default());
        self.translate_statements(false_br)?;
        let mut false_br = self.code.swap(before);

        if !false_br.is_final() {
            false_br.mark_jmp();
            false_br.write(Bytecode::Branch(false_br.pc() + true_br.pc() + 1));
        }

        self.code.mark_jmp();
        self.code
            .write(Bytecode::BrTrue(self.code.pc() + false_br.pc() + 1));

        self.code.extend(false_br)?;
        self.code.extend(true_br)?;
        Ok(())
    }

    fn translate_loop(
        &mut self,
        id: BlockId,
        cnd_calc: &[Statement],
        cnd: &Expression,
        // false br
        body: &[Statement],
    ) -> Result<(), Error> {
        dbg!(&self.code);

        self.code.create_label(id);
        self.translate_statements(cnd_calc)?;
        self.translate_expr(cnd)?;
        dbg!(&self.code);

        let before = self.code.swap(Writer::default());
        self.translate_statements(body)?;
        let loop_br = self.code.swap(before);
        dbg!(&loop_br);

        self.code.mark_jmp();
        self.code
            .write(Bytecode::BrTrue(self.code.pc() + loop_br.pc() + 1));
        self.code.extend(loop_br)?;
        Ok(())
    }
}
