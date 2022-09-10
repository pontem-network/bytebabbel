use crate::mv_ir::func::Func;
use crate::mv_ir::Module;
use crate::translator::signature::{map_signature, SignatureWriter};
use crate::translator::writer::{CallOp, Writer};
use anyhow::{anyhow, bail, Error};
use eth::abi::entries::FunHash;
use eth::bytecode::block::BlockId;
use eth::bytecode::mir::ir::expression::{Cast, Expression, StackOp};
use eth::bytecode::mir::ir::math::Operation;
use eth::bytecode::mir::ir::statement::Statement;
use eth::bytecode::mir::ir::types::{SType, Value};
use eth::bytecode::mir::ir::Mir;
use eth::bytecode::mir::translation::variables::Variable;
use eth::bytecode::types::EthType;
use eth::program::Program;
use intrinsic::{template, Mem, Num, Persist};
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
                SType::Num => Num::token(),
                SType::Bool => SignatureToken::Bool,
                SType::Storage => Persist::token(),
                SType::Memory => Mem::token(),
                SType::Address => SignatureToken::Reference(Box::new(SignatureToken::Signer)),
                SType::Bytes => SignatureToken::Vector(Box::new(SignatureToken::U8)),
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
                    Mem::Store,
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
                    Mem::Store8,
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
                    Persist::Store,
                    &[
                        CallOp::Var(*storage),
                        CallOp::Var(*offset),
                        CallOp::Var(*val),
                    ],
                );
            }
            Statement::InitStorage(var) => {
                self.code.call(Persist::InitContract, &[CallOp::Var(*var)]);
            }
            Statement::Log {
                storage,
                memory,
                offset,
                len,
                topics,
            } => {
                self.translate_log(*storage, *memory, *offset, *len, topics)?;
            }
        }
        Ok(())
    }

    fn translate_expr(&mut self, exp: &Expression) -> Result<(), Error> {
        match exp {
            Expression::Const(val) => {
                match val {
                    Value::Number(val) => {
                        let parts = val.0;
                        self.code.call(
                            Num::FromU64s,
                            &[
                                CallOp::ConstU64(parts[0]),
                                CallOp::ConstU64(parts[1]),
                                CallOp::ConstU64(parts[2]),
                                CallOp::ConstU64(parts[3]),
                            ],
                        );
                    }
                    Value::Bool(val) => {
                        if *val {
                            self.code.write(Bytecode::LdTrue);
                        } else {
                            self.code.write(Bytecode::LdFalse);
                        }
                    }
                };
            }
            Expression::Var(var) => {
                self.code.ld_var(var.index());
            }
            Expression::Operation(cmd, op, op1) => self.translate_operation(*cmd, op, op1),
            Expression::StackOps(ops) => {
                for op in &ops.vec {
                    match op {
                        StackOp::PushBoolVar(var) => {
                            self.code.ld_var(var.index());
                        }
                        StackOp::Not => {
                            self.code.write(Bytecode::Not);
                        }
                        StackOp::PushBool(val) => {
                            if *val {
                                self.code.write(Bytecode::LdTrue);
                            } else {
                                self.code.write(Bytecode::LdFalse);
                            }
                        }
                        StackOp::Eq => {
                            self.code.write(Bytecode::Eq);
                        }
                    }
                }
            }
            Expression::GetMem => {
                self.code
                    .call(Mem::New, &[CallOp::ConstU64(self.max_memory)]);
            }
            Expression::GetStore => {
                self.code
                    .write(Bytecode::LdConst(intrinsic::self_address_index()));
                self.code
                    .write(Bytecode::MutBorrowGlobal(Persist::instance()));
            }
            Expression::MLoad { memory, offset } => {
                self.code.call(
                    Mem::Load,
                    &[CallOp::MutBorrow(*memory), CallOp::Var(*offset)],
                );
            }
            Expression::SLoad { storage, offset } => {
                self.code.call(
                    Persist::Load,
                    &[CallOp::Var(*storage), CallOp::Var(*offset)],
                );
            }
            Expression::MSize { memory } => {
                self.code.call(Mem::Size, &[CallOp::MutBorrow(*memory)]);
            }
            Expression::MSlice {
                memory,
                offset,
                len,
            } => {
                self.code.call(
                    Mem::Slice,
                    &[
                        CallOp::MutBorrow(*memory),
                        CallOp::Var(*offset),
                        CallOp::Var(*len),
                    ],
                );
            }
            Expression::Cast(var, cast) => self.translate_cast(var, cast)?,
            Expression::BytesLen(bytes) => {
                self.code.call(Mem::BytesLen, &[CallOp::Borrow(*bytes)]);
            }
            Expression::ReadNum { data, offset } => {
                self.code.call(
                    Num::FromBytes,
                    &[CallOp::Borrow(*data), CallOp::Var(*offset)],
                );
            }
            Expression::Hash { mem, offset, len } => {
                self.code.call(
                    Mem::Hash,
                    &[
                        CallOp::MutBorrow(*mem),
                        CallOp::Var(*offset),
                        CallOp::Var(*len),
                    ],
                );
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

    fn translate_cast(&mut self, var: &Variable, cast: &Cast) -> Result<(), Error> {
        match cast {
            Cast::BoolToNum => self.code.call(Num::FromBool, &[CallOp::Var(*var)]),
            Cast::AddressToNum => self.code.call(Num::FromAddress, &[CallOp::Var(*var)]),
            Cast::BytesToNum => {
                self.code
                    .call(Num::FromBytes, &[CallOp::Var(*var), CallOp::ConstU64(0)]);
            }
            Cast::NumToBool => {
                self.code.call(Num::ToBool, &[CallOp::Var(*var)]);
            }
        }
        Ok(())
    }

    fn translate_log(
        &mut self,
        storage: Variable,
        memory: Variable,
        offset: Variable,
        len: Variable,
        topics: &[Variable],
    ) -> Result<(), Error> {
        let mut args = vec![
            CallOp::Var(storage),
            CallOp::MutBorrow(memory),
            CallOp::Var(offset),
            CallOp::Var(len),
        ];
        let fun = match topics.len() {
            0 => Persist::Log0,
            1 => {
                args.push(CallOp::Var(topics[0]));
                Persist::Log1
            }
            2 => {
                args.push(CallOp::Var(topics[0]));
                args.push(CallOp::Var(topics[1]));
                Persist::Log2
            }
            3 => {
                args.push(CallOp::Var(topics[0]));
                args.push(CallOp::Var(topics[1]));
                args.push(CallOp::Var(topics[2]));
                Persist::Log3
            }
            4 => {
                args.push(CallOp::Var(topics[0]));
                args.push(CallOp::Var(topics[1]));
                args.push(CallOp::Var(topics[2]));
                args.push(CallOp::Var(topics[3]));
                Persist::Log4
            }
            _ => bail!("too many topics"),
        };
        self.code.call(fun, &args);
        Ok(())
    }

    fn translate_operation(&mut self, operation: Operation, op1: &Variable, op2: &Variable) {
        let ops = [CallOp::Var(*op1), CallOp::Var(*op2)];
        match operation {
            Operation::Add => self.code.call(Num::Add, &ops),
            Operation::Sub => self.code.call(Num::Sub, &ops),
            Operation::Mul => self.code.call(Num::Mul, &ops),
            Operation::Eq => self.code.call(Num::Eq, &ops),
            Operation::Lt => self.code.call(Num::Lt, &ops),
            Operation::Gt => self.code.call(Num::Gt, &ops),
            Operation::Shr => self.code.call(Num::Shr, &ops),
            Operation::Shl => self.code.call(Num::Shl, &ops),
            Operation::Sar => self.code.call(Num::Sar, &ops),
            Operation::BitAnd => self.code.call(Num::BitAnd, &ops),
            Operation::BitOr => self.code.call(Num::BitOr, &ops),
            Operation::BitXor => self.code.call(Num::BitXor, &ops),
            Operation::Div => self.code.call(Num::Div, &ops),
            Operation::Byte => self.code.call(Num::Byte, &ops),
            Operation::Mod => self.code.call(Num::Mod, &ops),
            Operation::SDiv => self.code.call(Num::SDiv, &ops),
            Operation::SLt => self.code.call(Num::SLt, &ops),
            Operation::SGt => self.code.call(Num::SGt, &ops),
            Operation::SMod => self.code.call(Num::SMod, &ops),
            Operation::Exp => self.code.call(Num::Exp, &ops),
            Operation::SignExtend => self.code.call(Num::SignExtend, &ops),
            Operation::IsZero => self.code.call(Num::IsZero, &ops),
            Operation::BitNot => self.code.call(Num::BitNot, &ops),
        }
    }
}
