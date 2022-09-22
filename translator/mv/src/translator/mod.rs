use crate::mv_ir::func::Func;
use crate::mv_ir::Module;
use crate::translator::signature::{map_signature, signer, SignatureWriter};
use crate::translator::writer::{CallOp, Writer};
use anyhow::{anyhow, bail, Error};
use eth::abi::call::FunHash;
use eth::bytecode::block::BlockId;
use eth::bytecode::hir::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use eth::bytecode::mir::ir::expression::{Cast, Expression, StackOp};
use eth::bytecode::mir::ir::statement::Statement;
use eth::bytecode::mir::ir::types::{SType, Value};
use eth::bytecode::mir::ir::Mir;
use eth::bytecode::mir::translation::variables::Variable;
use eth::bytecode::types::EthType;
use eth::program::Program;
use eth::Flags;
use intrinsic::table::{Memory as Mem, Persist, U256 as Num};
use intrinsic::template;
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
    program: Option<Program>,
    flags: Flags,
}

impl MvIrTranslator {
    pub fn new(
        address: AccountAddress,
        max_memory: u64,
        program: Program,
        flags: Flags,
    ) -> MvIrTranslator {
        let template = template(address, program.name(), program.identifiers());
        Self {
            sign_writer: SignatureWriter::new(&template.signatures),
            code: Default::default(),
            template,
            max_memory,
            program: Some(program),
            flags,
        }
    }

    pub fn translate(mut self) -> Result<Module, Error> {
        let program = self.program.take().unwrap();

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

        let input =
            self.sign_writer
                .make_signature(map_signature(&[EthType::Address], false, &self.flags));
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

        let input = if self.flags.native_input {
            let mut input = vec![signer()];
            input.extend(map_signature(&def.native_input, true, &self.flags));
            self.sign_writer.make_signature(input)
        } else {
            self.sign_writer
                .make_signature(map_signature(&def.eth_input, false, &self.flags))
        };

        let output = if self.flags.hidden_output {
            self.sign_writer.make_signature(vec![])
        } else if self.flags.native_output {
            self.sign_writer
                .make_signature(map_signature(&def.native_output, true, &self.flags))
        } else {
            self.sign_writer
                .make_signature(map_signature(&def.eth_output, false, &self.flags))
        };

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
                SType::Signer => SignatureToken::Reference(Box::new(SignatureToken::Signer)),
                SType::Bytes => SignatureToken::Vector(Box::new(SignatureToken::U8)),
                SType::Address => SignatureToken::Address,
                SType::RawNum => SignatureToken::U128,
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
            Statement::Assign(var, exp) => {
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
                self.code
                    .call(Mem::RequestBufferLen, &[CallOp::Borrow(*bytes)]);
            }
            Expression::ReadNum { data, offset } => {
                self.code.call(
                    Mem::ReadRequestBuffer,
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
            Expression::Unary(op, arg) => self.translate_unary(op, arg)?,
            Expression::Binary(op, arg, arg1) => self.translate_binary(op, arg, arg1)?,
            Expression::Ternary(op, arg, arg1, arg2) => {
                self.translate_ternary(op, arg, arg1, arg2)?
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
            Cast::SignerToNum => self.code.call(Num::FromSigner, &[CallOp::Var(*var)]),
            Cast::BytesToNum => {
                self.code
                    .call(Num::FromBytes, &[CallOp::Var(*var), CallOp::ConstU64(0)]);
            }
            Cast::NumToBool => {
                self.code.call(Num::ToBool, &[CallOp::Var(*var)]);
            }
            Cast::AddressToNum => {
                self.code.call(Num::FromAddress, &[CallOp::Var(*var)]);
            }
            Cast::NumToAddress => {
                self.code.call(Num::ToAddress, &[CallOp::Var(*var)]);
            }
            Cast::RawNumToNum => {
                self.code.call(Num::FromU128, &[CallOp::Var(*var)]);
            }
            Cast::NumToRawNum => {
                self.code.call(Num::ToU128, &[CallOp::Var(*var)]);
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

    fn translate_unary(&mut self, op: &UnaryOp, arg: &Variable) -> Result<(), Error> {
        match op {
            UnaryOp::IsZero => {
                self.code.call(Num::IsZero, &[CallOp::Var(*arg)]);
            }
            UnaryOp::Not => {
                self.code.call(Num::BitNot, &[CallOp::Var(*arg)]);
            }
        }
        Ok(())
    }

    fn translate_binary(
        &mut self,
        op: &BinaryOp,
        arg: &Variable,
        arg1: &Variable,
    ) -> Result<(), Error> {
        let args = [CallOp::Var(*arg), CallOp::Var(*arg1)];
        let index = match op {
            BinaryOp::Eq => Num::Eq,
            BinaryOp::Lt => Num::Lt,
            BinaryOp::Gt => Num::Gt,
            BinaryOp::Shr => Num::Shr,
            BinaryOp::Shl => Num::Shl,
            BinaryOp::Sar => Num::Sar,
            BinaryOp::Add => Num::Add,
            BinaryOp::And => Num::BitAnd,
            BinaryOp::Or => Num::BitOr,
            BinaryOp::Xor => Num::BitXor,
            BinaryOp::Mul => Num::Mul,
            BinaryOp::Sub => Num::Sub,
            BinaryOp::Div => Num::Div,
            BinaryOp::SDiv => Num::SDiv,
            BinaryOp::SLt => Num::SLt,
            BinaryOp::SGt => Num::SGt,
            BinaryOp::Byte => Num::Byte,
            BinaryOp::Mod => Num::Mod,
            BinaryOp::SMod => Num::SMod,
            BinaryOp::Exp => Num::Exp,
            BinaryOp::SignExtend => Num::SignExtend,
        };
        self.code.call(index, &args);
        Ok(())
    }

    fn translate_ternary(
        &mut self,
        op: &TernaryOp,
        _arg: &Variable,
        _arg1: &Variable,
        _arg2: &Variable,
    ) -> Result<(), Error> {
        match op {
            TernaryOp::AddMod => {
                todo!()
            }
            TernaryOp::MulMod => {
                todo!()
            }
        }
    }
}
