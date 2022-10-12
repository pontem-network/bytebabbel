use anyhow::{anyhow, Error, Result};
use move_binary_format::file_format::{Bytecode, SignatureIndex, SignatureToken, Visibility};
use move_binary_format::CompiledModule;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use std::collections::BTreeMap;

use eth::abi::call::FunHash;
use eth::bytecode::hir::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use eth::bytecode::loc::Loc;
use eth::bytecode::mir::ir::expression::{Cast, Expression, TypedExpr};
use eth::bytecode::mir::ir::statement::Statement;
use eth::bytecode::mir::ir::types::{SType, Value};
use eth::bytecode::mir::ir::Mir;
use eth::bytecode::mir::translation::variables::Variable;
use eth::bytecode::types::EthType;
use eth::program::Program;
use eth::Flags;
use intrinsic::table::{self_address_index, Info, Memory as Mem, Persist, U256 as Num};
use intrinsic::{template, Function};

use crate::mv_ir::func::Func;
use crate::mv_ir::Module;
use crate::translator::signature::{map_signature, signer, SignatureWriter};
use crate::translator::writer::Code;

pub mod bytecode;
pub mod signature;
pub mod writer;

pub struct MvIrTranslator {
    sign_writer: SignatureWriter,
    code: Code,
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
    ) -> Result<MvIrTranslator> {
        let template = template(address, program.name(), program.identifiers())?;
        Ok(Self {
            sign_writer: SignatureWriter::new(&template.signatures),
            code: Default::default(),
            template,
            max_memory,
            program: Some(program),
            flags,
        })
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
        self.translate_statements(mir.statements());
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
        self.translate_statements(mir.statements());
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

    fn translate_statements(&mut self, statements: &[Loc<Statement>]) {
        for st in statements {
            self.translate_statement(st);
        }
    }

    fn translate_statement(&mut self, st: &Loc<Statement>) {
        match st.as_ref() {
            Statement::Assign(var, exp) => {
                self.translate_expr(exp);
                self.code.assign(var.index());
            }
            Statement::Abort(code) => {
                self.code.abort(*code);
            }
            Statement::Result(vars) => {
                for var in vars {
                    self.code.move_loc(var.index());
                }
                self.code.write(Bytecode::Ret);
            }
            Statement::MStore {
                memory,
                offset,
                val,
            } => {
                self.call(
                    Mem::Store,
                    vec![
                        CallOp::MutBorrow(*memory),
                        CallOp::Expr(offset),
                        CallOp::Expr(val),
                    ],
                );
            }
            Statement::MStore8 {
                memory,
                offset,
                val,
            } => {
                self.call(
                    Mem::Store8,
                    vec![
                        CallOp::MutBorrow(*memory),
                        CallOp::Expr(offset),
                        CallOp::Expr(val),
                    ],
                );
            }
            Statement::SStore { storage, key, val } => {
                self.call(
                    Persist::Store,
                    vec![CallOp::Copy(*storage), CallOp::Expr(key), CallOp::Expr(val)],
                );
            }
            Statement::InitStorage(var) => {
                self.call(Persist::InitContract, vec![CallOp::Copy(*var)]);
            }
            Statement::Log {
                storage,
                memory,
                offset,
                len,
                topics,
            } => {
                self.translate_log(*storage, *memory, offset, len, topics);
            }
            Statement::StoreStack(ctx) => {
                self.translate_store_stack(ctx);
            }
            Statement::Label(lbl) => {
                self.code.label(*lbl);
            }
            Statement::BrTrue(cnd, goto) => {
                self.translate_expr(cnd);
                self.code.jmp(*goto, true);
            }
            Statement::Br(goto) => {
                self.code.jmp(*goto, false);
            }
        }
    }

    fn translate_store_stack(&mut self, ctx: &BTreeMap<Variable, Loc<TypedExpr>>) {
        let mut st_locs = Vec::with_capacity(ctx.len());

        for (var, loc) in ctx {
            st_locs.push(var);
            self.translate_expr(loc);
        }

        for var in st_locs.iter().rev() {
            self.code.assign(var.index());
        }
    }

    fn translate_expr(&mut self, exp: &Loc<TypedExpr>) {
        match &*exp.expr {
            Expression::Const(val) => {
                match val {
                    Value::Number(val) => {
                        let parts = val.0;
                        self.call(
                            Num::FromU64s,
                            vec![
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
            Expression::MoveVar(var) => {
                self.code.move_loc(var.index());
            }
            Expression::CopyVar(var) => {
                self.code.copy_loc(var.index());
            }
            Expression::GetMem => {
                self.call(Mem::New, vec![CallOp::ConstU64(self.max_memory)]);
            }
            Expression::GetStore => {
                let index = self_address_index();
                self.code.write(Bytecode::LdConst(index));

                self.code
                    .write(Bytecode::MutBorrowGlobal(Persist::instance()));
            }
            Expression::MLoad { memory, offset } => {
                self.call(
                    Mem::Load,
                    vec![CallOp::MutBorrow(*memory), CallOp::Expr(offset)],
                );
            }
            Expression::SLoad { storage, key } => {
                self.call(
                    Persist::Load,
                    vec![CallOp::Copy(*storage), CallOp::Expr(key)],
                );
            }
            Expression::MSize { memory } => {
                self.call(Mem::Size, vec![CallOp::MutBorrow(*memory)]);
            }
            Expression::MSlice {
                memory,
                offset,
                len,
            } => {
                self.call(
                    Mem::Slice,
                    vec![
                        CallOp::MutBorrow(*memory),
                        CallOp::Expr(offset),
                        CallOp::Expr(len),
                    ],
                );
            }
            Expression::Cast(var, cast) => self.translate_cast(var, cast),
            Expression::BytesLen(bytes) => {
                self.call(Mem::RequestBufferLen, vec![CallOp::Borrow(*bytes)]);
            }
            Expression::ReadNum { data, offset } => {
                self.call(
                    Mem::ReadRequestBuffer,
                    vec![CallOp::Borrow(*data), CallOp::Expr(offset)],
                );
            }
            Expression::Hash { mem, offset, len } => {
                self.call(
                    Mem::Hash,
                    vec![
                        CallOp::MutBorrow(*mem),
                        CallOp::Expr(offset),
                        CallOp::Expr(len),
                    ],
                );
            }
            Expression::Unary(op, arg) => self.translate_unary(*op, arg),
            Expression::Binary(op, arg, arg1) => self.translate_binary(*op, arg, arg1),
            Expression::Ternary(op, arg, arg1, arg2) => {
                self.translate_ternary(*op, arg, arg1, arg2)
            }
            Expression::Balance { address } => {
                self.code
                    .call(Info::AptosBalance, vec![CallOp::Var(*address)]);
            }
            Expression::GasPrice => {
                self.code.call(Info::GasPrice, vec![]);
            }
        }
    }

    fn translate_cast(&mut self, arg: &Loc<TypedExpr>, cast: &Cast) {
        let arg = CallOp::Expr(arg);
        match cast {
            Cast::BoolToNum => self.call(Num::FromBool, vec![arg]),
            Cast::SignerToNum => self.call(Num::FromSigner, vec![arg]),
            Cast::BytesToNum => {
                self.call(Num::FromBytes, vec![arg, CallOp::ConstU64(0)]);
            }
            Cast::NumToBool => {
                self.call(Num::ToBool, vec![arg]);
            }
            Cast::AddressToNum => {
                self.call(Num::FromAddress, vec![arg]);
            }
            Cast::NumToAddress => {
                self.call(Num::ToAddress, vec![arg]);
            }
            Cast::RawNumToNum => {
                self.call(Num::FromU128, vec![arg]);
            }
            Cast::NumToRawNum => {
                self.call(Num::ToU128, vec![arg]);
            }
        }
    }

    fn translate_log(
        &mut self,
        storage: Variable,
        memory: Variable,
        offset: &Loc<TypedExpr>,
        len: &Loc<TypedExpr>,
        topics: &[Loc<TypedExpr>],
    ) {
        let fun = match topics.len() {
            0 => Persist::Log0,
            1 => Persist::Log1,
            2 => Persist::Log2,
            3 => Persist::Log3,
            4 => Persist::Log4,
            _ => panic!("too many topics"),
        };

        let mut args = vec![
            CallOp::Copy(storage),
            CallOp::MutBorrow(memory),
            CallOp::Expr(offset),
            CallOp::Expr(len),
        ];

        topics
            .iter()
            .map(CallOp::Expr)
            .for_each(|arg| args.push(arg));
        self.call(fun, args);
    }

    fn translate_unary(&mut self, op: UnaryOp, arg: &Loc<TypedExpr>) {
        let args = vec![CallOp::Expr(arg)];
        match op {
            UnaryOp::IsZero => {
                self.call(Num::IsZero, args);
            }
            UnaryOp::Not => {
                self.call(Num::BitNot, args);
            }
        }
    }

    fn translate_binary(&mut self, op: BinaryOp, arg: &Loc<TypedExpr>, arg1: &Loc<TypedExpr>) {
        if op == BinaryOp::Eq || arg.ty == SType::Bool && arg1.ty == SType::Bool {
            self.translate_expr(arg);
            self.translate_expr(arg1);
            self.code.write(Bytecode::Eq);
            return;
        }

        let args = vec![CallOp::Expr(arg), CallOp::Expr(arg1)];
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
        self.call(index, args)
    }

    fn translate_ternary(
        &mut self,
        op: TernaryOp,
        arg: &Loc<TypedExpr>,
        arg1: &Loc<TypedExpr>,
        arg2: &Loc<TypedExpr>,
    ) {
        let args = vec![CallOp::Expr(arg), CallOp::Expr(arg1), CallOp::Expr(arg2)];
        let index = match op {
            TernaryOp::AddMod => Num::AddMod,
            TernaryOp::MulMod => Num::MulMod,
        };
        self.call(index, args);
    }

    fn call(&mut self, fun: impl Function, args: Vec<CallOp>) {
        for arg in args {
            match arg {
                CallOp::Move(var) => {
                    self.code.move_loc(var.index());
                }
                CallOp::ConstU64(val) => {
                    self.code.write(Bytecode::LdU64(val));
                }
                CallOp::MutBorrow(var) => {
                    self.code.write(Bytecode::MutBorrowLoc(var.index()));
                }
                CallOp::Borrow(var) => {
                    self.code.write(Bytecode::ImmBorrowLoc(var.index()));
                }
                CallOp::Expr(code) => {
                    self.translate_expr(code);
                }
                CallOp::Copy(var) => {
                    self.code.copy_loc(var.index());
                }
            }
        }
        self.code.write(Bytecode::Call(fun.handler()));
    }
}

#[derive(Debug)]
pub enum CallOp<'a> {
    Expr(&'a Loc<TypedExpr>),
    Move(Variable),
    Copy(Variable),
    MutBorrow(Variable),
    Borrow(Variable),
    ConstU64(u64),
}
