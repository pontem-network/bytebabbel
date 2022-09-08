use crate::mv_ir::func::Func;
use crate::mv_ir::Module;
use crate::translator::signature::{map_signature, SignatureWriter};
use crate::translator::writer::{CallOp, Writer};
use anyhow::{anyhow, Error};
use eth::abi::entries::FunHash;
use eth::bytecode::block::BlockId;
use eth::bytecode::mir::ir::expression::{Expression, StackOp};
use eth::bytecode::mir::ir::math::Operation;
use eth::bytecode::mir::ir::statement::Statement;
use eth::bytecode::mir::ir::types::SType;
use eth::bytecode::mir::ir::Mir;
use eth::bytecode::types::EthType;
use eth::program::Program;
use intrinsic::{template, Mem, Storage};
use move_binary_format::file_format::{Bytecode, SignatureIndex, SignatureToken, Visibility};
use move_binary_format::CompiledModule;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;

pub mod bytecode;
pub mod signature;
pub mod writer;

pub struct MvIrTranslator {
    sign_writer: SignatureWriter,
    code: Writer,
    template: CompiledModule,
    max_memory: u64,
}

impl MvIrTranslator {
    pub fn new(address: AccountAddress, name: &str) -> MvIrTranslator {
        let template = template(address, name);
        Self {
            sign_writer: SignatureWriter::new(&template.signatures),
            code: Default::default(),
            template,
            max_memory: 0,
        }
    }

    pub fn translate(mut self, max_memory: u64, program: Program) -> Result<Module, Error> {
        self.max_memory = max_memory;
        let mut funcs = program
            .functions_hash()
            .into_iter()
            .map(|hash| self.translate_func(hash, &program))
            .collect::<Result<Vec<_>, _>>()?;

        funcs.push(self.translate_constructor(&program)?);

        Ok(Module::new(funcs, self.sign_writer.freeze(), self.template))
    }

    fn translate_constructor(&mut self, program: &Program) -> Result<Func, Error> {
        let mir = program.constructor_mir().clone();

        self.code.reset();
        self.translate_statements(mir.statements())?;
        let code = self.code.freeze()?;

        let input = self
            .sign_writer
            .make_signature(map_signature(&[EthType::Address]));
        let output = self.sign_writer.make_signature(vec![]);
        Ok(Func {
            name: Identifier::new("constructor")?,
            visibility: Visibility::Public,
            input,
            output,
            locals: self.map_locals(&mir),
            code,
        })
    }

    fn translate_func(&mut self, hash: FunHash, program: &Program) -> Result<Func, Error> {
        let def = program.function_def(hash).ok_or_else(|| {
            anyhow!(
                "Function with hash {} not found in program {}",
                hash,
                program.name()
            )
        })?;
        let mir = program.function_mir(hash).ok_or_else(|| {
            anyhow!(
                "Function with hash {} not found in program {}",
                hash,
                program.name()
            )
        })?;

        let name = Identifier::new(def.name.clone())?;
        let visibility = Visibility::Public;

        let input = self.sign_writer.make_signature(map_signature(&def.input));

        let output = self.sign_writer.make_signature(map_signature(&def.output));

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
                SType::Number => SignatureToken::U128,
                SType::Bool => SignatureToken::Bool,
                SType::Storage => Storage::token(),
                SType::Memory => Mem::token(),
                SType::Address => SignatureToken::Reference(Box::new(SignatureToken::Signer)),
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
            Statement::MStore {
                memory,
                offset,
                val,
            } => {
                self.code.call(
                    Mem::Store.func_handler(),
                    &[
                        CallOp::MutBorrow(*memory),
                        CallOp::Var(*offset),
                        CallOp::Var(*val),
                    ],
                );
            }
            Statement::MStore8 {
                memory,
                offset,
                val,
            } => {
                self.code.call(
                    Mem::Store8.func_handler(),
                    &[
                        CallOp::MutBorrow(*memory),
                        CallOp::Var(*offset),
                        CallOp::Var(*val),
                    ],
                );
            }
            Statement::SStore {
                storage,
                key: offset,
                val,
            } => {
                self.code.call(
                    Storage::Store.func_handler(),
                    &[
                        CallOp::Var(*storage),
                        CallOp::Var(*offset),
                        CallOp::Var(*val),
                    ],
                );
            }
            Statement::InitStorage(var) => {
                self.code
                    .call(Storage::Create.func_handler(), &[CallOp::Var(*var)]);
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
            Expression::GetMem => {
                self.code.call(
                    Mem::New.func_handler(),
                    &[CallOp::ConstU64(self.max_memory)],
                );
            }
            Expression::GetStore => {
                self.code
                    .write(Bytecode::LdConst(intrinsic::self_address_index()));
                self.code
                    .write(Bytecode::MutBorrowGlobal(Storage::instance()));
            }
            Expression::MLoad { memory, offset } => {
                self.code.call(
                    Mem::Load.func_handler(),
                    &[CallOp::MutBorrow(*memory), CallOp::Var(*offset)],
                );
            }
            Expression::SLoad { storage, offset } => {
                self.code.call(
                    Storage::Load.func_handler(),
                    &[CallOp::Var(*storage), CallOp::Var(*offset)],
                );
            }
            Expression::MSize { memory } => {
                self.code
                    .call(Mem::Size.func_handler(), &[CallOp::MutBorrow(*memory)]);
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
        self.code.create_label(id);
        self.translate_statements(cnd_calc)?;
        self.translate_expr(cnd)?;

        let before = self.code.swap(Writer::default());
        self.translate_statements(body)?;
        let loop_br = self.code.swap(before);

        self.code.mark_jmp();
        self.code
            .write(Bytecode::BrTrue(self.code.pc() + loop_br.pc() + 1));
        self.code.extend(loop_br)?;
        Ok(())
    }
}
