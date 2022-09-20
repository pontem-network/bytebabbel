use crate::module::func::Func;
use crate::module::Module;
use crate::translator::signature::{map_signature, signer, SignatureWriter};
use crate::translator::writer::{CallOp, Code};
use anyhow::{anyhow, bail, Error};
use eth::abi::entries::FunHash;
use eth::bytecode::block::BlockId;
use eth::bytecode::hir::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use eth::bytecode::mir::ir::expression::{Cast, Expression, StackOp, TypedExpr};
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

pub struct MvTranslator {
    sign_writer: SignatureWriter,
    code: Code,
    template: CompiledModule,
    max_memory: u64,
    program: Option<Program>,
    flags: Flags,
}

impl MvTranslator {
    pub fn new(
        address: AccountAddress,
        max_memory: u64,
        program: Program,
        flags: Flags,
    ) -> MvTranslator {
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
                self.expr(exp)?;
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
                    self.expr(var)?;
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
                let offset = self.call_op(offset)?;
                let val = self.call_op(val)?;
                self.code
                    .call(Mem::Store, &[CallOp::MutBorrow(*memory), offset, val]);
            }
            Statement::MStore8 {
                memory,
                offset,
                val,
            } => {
                let offset = self.call_op(offset)?;
                let val = self.call_op(val)?;
                self.code
                    .call(Mem::Store8, &[CallOp::MutBorrow(*memory), offset, val]);
            }
            Statement::SStore { storage, key, val } => {
                let key = self.call_op(key)?;
                let val = self.call_op(val)?;
                self.code
                    .call(Persist::Store, &[CallOp::Var(*storage), key, val]);
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
                self.translate_log(*storage, *memory, offset, len, topics)?;
            }
        }
        Ok(())
    }

    fn expr(&mut self, exp: &TypedExpr) -> Result<(), Error> {
        match exp.expr.as_ref() {
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
                        StackOp::PushBoolExpr(var) => {
                            self.expr(var)?;
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
                let offset = self.call_op(offset)?;
                self.code
                    .call(Mem::Load, &[CallOp::MutBorrow(*memory), offset]);
            }
            Expression::SLoad { storage, key } => {
                let key = self.call_op(key)?;
                self.code.call(Persist::Load, &[CallOp::Var(*storage), key]);
            }
            Expression::MSize { memory } => {
                self.code.call(Mem::Size, &[CallOp::MutBorrow(*memory)]);
            }
            Expression::MSlice {
                memory,
                offset,
                len,
            } => {
                let offset = self.call_op(offset)?;
                let len = self.call_op(len)?;
                self.code
                    .call(Mem::Slice, &[CallOp::MutBorrow(*memory), offset, len]);
            }
            Expression::Cast(var, cast) => self.translate_cast(var, cast)?,
            Expression::BytesLen(bytes) => {
                self.code
                    .call(Mem::RequestBufferLen, &[CallOp::Borrow(*bytes)]);
            }
            Expression::ReadNum { data, offset } => {
                let offset = self.call_op(offset)?;
                self.code
                    .call(Mem::ReadRequestBuffer, &[CallOp::Borrow(*data), offset]);
            }
            Expression::Hash { mem, offset, len } => {
                let offset = self.call_op(offset)?;
                let len = self.call_op(len)?;
                self.code
                    .call(Mem::Hash, &[CallOp::MutBorrow(*mem), offset, len]);
            }
            Expression::UnOp(cmd, op) => {
                self.translate_unary(*cmd, op)?;
            }
            Expression::BinOp(cmd, op, op1) => {
                self.translate_binary(*cmd, op, op1)?;
            }
            Expression::TernOp(cmd, op, op1, op2) => {
                self.translate_ternary(*cmd, op, op1, op2)?;
            }
        }
        Ok(())
    }

    fn translate_if(
        &mut self,
        cnd: &TypedExpr,
        true_br: &[Statement],
        false_br: &[Statement],
    ) -> Result<(), Error> {
        self.expr(cnd)?;
        let before = self.code.swap(Code::default());
        self.translate_statements(true_br)?;
        let true_br = self.code.swap(Code::default());
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
        cnd: &TypedExpr,
        // false br
        body: &[Statement],
    ) -> Result<(), Error> {
        self.code.create_label(id);
        self.translate_statements(cnd_calc)?;
        self.expr(cnd)?;

        let before = self.code.swap(Code::default());
        self.translate_statements(body)?;
        let loop_br = self.code.swap(before);

        self.code.mark_jmp();
        self.code
            .write(Bytecode::BrTrue(self.code.pc() + loop_br.pc() + 1));
        self.code.extend(loop_br)?;
        Ok(())
    }

    fn translate_cast(&mut self, var: &TypedExpr, cast: &Cast) -> Result<(), Error> {
        let var = self.call_op(var)?;
        match cast {
            Cast::BoolToNum => self.code.call(Num::FromBool, &[var]),
            Cast::SignerToNum => self.code.call(Num::FromSigner, &[var]),
            Cast::BytesToNum => {
                self.code.call(Num::FromBytes, &[var, CallOp::ConstU64(0)]);
            }
            Cast::NumToBool => {
                self.code.call(Num::ToBool, &[var]);
            }
            Cast::AddressToNum => {
                self.code.call(Num::FromAddress, &[var]);
            }
            Cast::NumToAddress => {
                self.code.call(Num::ToAddress, &[var]);
            }
            Cast::RawNumToNum => {
                self.code.call(Num::FromU128, &[var]);
            }
            Cast::NumToRawNum => {
                self.code.call(Num::ToU128, &[var]);
            }
        }
        Ok(())
    }

    fn translate_log(
        &mut self,
        storage: Variable,
        memory: Variable,
        offset: &TypedExpr,
        len: &TypedExpr,
        topics: &[TypedExpr],
    ) -> Result<(), Error> {
        let mut args = vec![
            CallOp::Var(storage),
            CallOp::MutBorrow(memory),
            self.call_op(offset)?,
            self.call_op(len)?,
        ];
        let fun = match topics.len() {
            0 => Persist::Log0,
            1 => {
                args.push(self.call_op(&topics[0])?);
                Persist::Log1
            }
            2 => {
                args.push(self.call_op(&topics[0])?);
                args.push(self.call_op(&topics[1])?);
                Persist::Log2
            }
            3 => {
                args.push(self.call_op(&topics[0])?);
                args.push(self.call_op(&topics[1])?);
                args.push(self.call_op(&topics[2])?);
                Persist::Log3
            }
            4 => {
                args.push(self.call_op(&topics[0])?);
                args.push(self.call_op(&topics[1])?);
                args.push(self.call_op(&topics[2])?);
                args.push(self.call_op(&topics[3])?);
                Persist::Log4
            }
            _ => bail!("too many topics"),
        };
        self.code.call(fun, &args);
        Ok(())
    }

    fn translate_unary(&mut self, op: UnaryOp, expr: &TypedExpr) -> Result<(), Error> {
        let expr = self.call_op(expr)?;
        match op {
            UnaryOp::Not => self.code.call(Num::BitNot, &[expr]),
            UnaryOp::IsZero => self.code.call(Num::IsZero, &[expr]),
        }
        Ok(())
    }

    fn translate_binary(
        &mut self,
        cmd: BinaryOp,
        op1: &TypedExpr,
        op2: &TypedExpr,
    ) -> Result<(), Error> {
        let op1 = self.call_op(op1)?;
        let op2 = self.call_op(op2)?;
        let ops = [op1, op2];
        match cmd {
            BinaryOp::Add => self.code.call(Num::Add, &ops),
            BinaryOp::Sub => self.code.call(Num::Sub, &ops),
            BinaryOp::Mul => self.code.call(Num::Mul, &ops),
            BinaryOp::Eq => self.code.call(Num::Eq, &ops),
            BinaryOp::Lt => self.code.call(Num::Lt, &ops),
            BinaryOp::Gt => self.code.call(Num::Gt, &ops),
            BinaryOp::Shr => self.code.call(Num::Shr, &ops),
            BinaryOp::Shl => self.code.call(Num::Shl, &ops),
            BinaryOp::Sar => self.code.call(Num::Sar, &ops),
            BinaryOp::And => self.code.call(Num::BitAnd, &ops),
            BinaryOp::Or => self.code.call(Num::BitOr, &ops),
            BinaryOp::Xor => self.code.call(Num::BitXor, &ops),
            BinaryOp::Div => self.code.call(Num::Div, &ops),
            BinaryOp::Byte => self.code.call(Num::Byte, &ops),
            BinaryOp::Mod => self.code.call(Num::Mod, &ops),
            BinaryOp::SDiv => self.code.call(Num::SDiv, &ops),
            BinaryOp::SLt => self.code.call(Num::SLt, &ops),
            BinaryOp::SGt => self.code.call(Num::SGt, &ops),
            BinaryOp::SMod => self.code.call(Num::SMod, &ops),
            BinaryOp::Exp => self.code.call(Num::Exp, &ops),
            BinaryOp::SignExtend => self.code.call(Num::SignExtend, &ops),
        }
        Ok(())
    }

    fn translate_ternary(
        &mut self,
        cmd: TernaryOp,
        op1: &TypedExpr,
        op2: &TypedExpr,
        op3: &TypedExpr,
    ) -> Result<(), Error> {
        let op1 = self.call_op(op1)?;
        let op2 = self.call_op(op2)?;
        let op3 = self.call_op(op3)?;
        let _ops = [op1, op2, op3];
        match cmd {
            TernaryOp::AddMod => todo!(),
            TernaryOp::MulMod => todo!(),
        }
    }

    fn call_op(&mut self, expr: &TypedExpr) -> Result<CallOp, Error> {
        let old_code = self.code.swap(Code::default());
        self.expr(expr)?;
        let expr_code = self.code.swap(old_code);
        Ok(CallOp::Expr(expr_code))
    }
}
