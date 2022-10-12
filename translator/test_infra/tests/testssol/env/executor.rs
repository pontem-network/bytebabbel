use std::collections::HashMap;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use itertools::Itertools;
use serde::Deserialize;

use aptos_aggregator::transaction::ChangeSetExt;
use aptos_crypto::HashValue;
use aptos_gas::{AbstractValueSizeGasParameters, NativeGasParameters};
use aptos_state_view::StateView;
use aptos_types::state_store::state_key::StateKey;
use aptos_types::state_store::state_storage_usage::StateStorageUsage;
use aptos_types::write_set::WriteOp;
use aptos_vm::data_cache::StorageAdapter;
use aptos_vm::move_vm_ext::{MoveVmExt, SessionId};
use aptos_vm::natives::configure_for_unit_test;
use ethabi::{Contract, Token};
use move_core_types::account_address::AccountAddress;
use move_core_types::effects::Event;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{ModuleId, CORE_CODE_ADDRESS};
use move_core_types::value::MoveTypeLayout;
use move_vm_runtime::session::LoadedFunctionInstantiation;
use move_vm_types::gas::UnmeteredGasMeter;
use move_vm_types::loaded_data::runtime_types::Type;
use once_cell::sync::OnceCell;
use primitive_types::{H160, U256};

use eth::abi::call::{to_eth_address, EthEncodeByString};
use eth::Flags;

use crate::testssol::env::stdlib::publish_std;

static INSTANCE: OnceCell<Resolver> = OnceCell::new();
const BALANCE_MV: &str =
    "./translator/test_infra/resources/mv/build/mv/bytecode_modules/balance.mv";

pub struct MoveExecutor {
    resolver: Resolver,
    vm: MoveVmExt,
    seq: u64,
    entries: Contract,
    flags: Flags,
}

impl MoveExecutor {
    pub fn new(entries: Contract, flags: Flags) -> MoveExecutor {
        let resolver = INSTANCE
            .get_or_init(|| {
                let mut resolver = Resolver::default();
                let vm = Self::create_vm();
                let id = SessionId::Txn {
                    sender: CORE_CODE_ADDRESS,
                    sequence_number: 0,
                    script_hash: vec![0; 32],
                };
                let adapter = StorageAdapter::new(&resolver);
                let mut session = vm.new_session(&adapter, id);
                publish_std(&mut session);

                // let t = fs::read(BALANCE_MV).unwrap();
                // todo!();

                // session.publish_module(fs::read())
                let output = session
                    .finish()
                    .unwrap()
                    .into_change_set(&mut (), 3)
                    .unwrap();
                resolver.apply(output);
                resolver
            })
            .clone();

        MoveExecutor {
            resolver,
            vm: Self::create_vm(),
            seq: 1,
            entries,
            flags,
        }
    }

    fn create_vm() -> MoveVmExt {
        configure_for_unit_test();
        MoveVmExt::new(
            NativeGasParameters::zeros(),
            AbstractValueSizeGasParameters::zeros(),
            3,
            false,
        )
        .unwrap()
    }

    pub fn deploy(&mut self, addr: &str, module: Vec<u8>) {
        let addr = AccountAddress::from_hex_literal(addr).unwrap();
        let id = SessionId::Txn {
            sender: addr,
            sequence_number: self.seq,
            script_hash: HashValue::sha3_256_of(&module).to_vec(),
        };
        self.seq += 1;

        let adapter = StorageAdapter::new(&self.resolver);
        let mut session = self.vm.new_session(&adapter, id);
        session
            .publish_module(module, addr, &mut UnmeteredGasMeter)
            .unwrap();
        let output = session
            .finish()
            .unwrap()
            .into_change_set(&mut (), 3)
            .unwrap();
        self.resolver.apply(output);
    }

    pub fn run(
        &mut self,
        ident: &str,
        signer: &str,
        params: Option<&str>,
    ) -> Result<ExecutionResult> {
        let (module_id, ident) = Self::prepare_ident(ident);
        let id = SessionId::Txn {
            sender: *module_id.address(),
            sequence_number: self.seq,
            script_hash: HashValue::sha3_256_of(params.unwrap_or_default().as_bytes()).to_vec(),
        };
        self.seq += 1;

        let adapter = StorageAdapter::new(&self.resolver);
        let mut session = self.vm.new_session(&adapter, id);
        let fn_name = ident.as_str();

        let args = if self.flags.native_input {
            let fun = session.load_function(&module_id, &ident, &[]);
            self.prepare_move_args(signer, params, &fun.unwrap())?
        } else {
            self.prepare_eth_args(signer, params, fn_name)?
        };
        let returns = session
            .execute_entry_function(&module_id, &ident, vec![], args, &mut UnmeteredGasMeter)?
            .return_values;
        let result = session.finish().unwrap();
        let events = result.events.clone();
        let output = result.into_change_set(&mut (), 3).unwrap();

        let returns = if self.flags.hidden_output {
            vec![]
        } else if self.flags.native_output {
            self.decode_result_move(returns)?
        } else {
            self.decode_result_eth(returns, fn_name)?
        };

        self.resolver.apply(output);

        Ok(ExecutionResult { returns, events })
    }

    fn decode_result_move(&self, result: Vec<(Vec<u8>, MoveTypeLayout)>) -> Result<Vec<Token>> {
        result
            .iter()
            .map(|(val, tp)| match tp {
                MoveTypeLayout::Bool => bcs::from_bytes::<bool>(val).map(Token::Bool),
                MoveTypeLayout::U8 => {
                    bcs::from_bytes::<u8>(val).map(|val| Token::Uint(U256::from(val)))
                }
                MoveTypeLayout::U64 => {
                    bcs::from_bytes::<u64>(val).map(|val| Token::Uint(U256::from(val)))
                }
                MoveTypeLayout::U128 => {
                    bcs::from_bytes::<u128>(val).map(|val| Token::Uint(U256::from(val)))
                }
                MoveTypeLayout::Address => bcs::from_bytes::<AccountAddress>(val)
                    .map(|val| Token::Address(H160::from(to_eth_address(val.as_ref())))),
                MoveTypeLayout::Vector(_) => {
                    todo!()
                }
                MoveTypeLayout::Struct(_) => {
                    bcs::from_bytes::<U256Wrapper>(val).map(|val| Token::Uint(U256(val.0)))
                }
                _ => unreachable!(),
            })
            .collect::<Result<Vec<Token>, _>>()
            .map_err(|e| anyhow!(e))
    }

    fn decode_result_eth(
        &self,
        result: Vec<(Vec<u8>, MoveTypeLayout)>,
        fn_name: &str,
    ) -> Result<Vec<Token>> {
        if fn_name == "constructor" {
            Ok(Vec::new())
        } else if !result.is_empty() {
            let result: Vec<u8> = bcs::from_bytes(&result[0].0).map_err(|e| anyhow!(e))?;
            let result = self
                .entries
                .functions_by_name(fn_name)?
                .first()
                .ok_or_else(|| anyhow!("Fn {fn_name:?} not found "))?
                .decode_output(&result)?;
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }

    fn prepare_ident(ident: &str) -> (ModuleId, Identifier) {
        let mut split = ident.split("::");
        let addr = AccountAddress::from_hex_literal(split.next().unwrap()).unwrap();
        let name = Identifier::from_str(split.next().unwrap()).unwrap();
        (
            ModuleId::new(addr, name),
            Identifier::from_str(split.next().unwrap()).unwrap(),
        )
    }

    fn prepare_eth_args(
        &self,
        signer: &str,
        args: Option<&str>,
        fn_name: &str,
    ) -> Result<Vec<Vec<u8>>> {
        let signer = bcs::to_bytes(&AccountAddress::from_hex_literal(signer).unwrap())?;

        if let Some(args) = args {
            let request = if fn_name == "constructor" {
                self.entries.constructor().map(|fun| fun.call_by_str(args))
            } else {
                self.entries
                    .functions_by_name(fn_name)?
                    .first()
                    .map(|fun| fun.call_by_str(args))
            };
            if let Some(req) = request {
                let request = req?;
                Ok(vec![signer, bcs::to_bytes(&request[4..])?])
            } else {
                Ok(vec![signer])
            }
        } else {
            Ok(vec![signer])
        }
    }

    fn prepare_move_args(
        &self,
        signer: &str,
        args: Option<&str>,
        fun: &LoadedFunctionInstantiation,
    ) -> Result<Vec<Vec<u8>>> {
        if let Some(args) = args {
            let res = format!("{},{}", signer, args)
                .split(',')
                .zip(&fun.parameters)
                .map(|(val, tp)| {
                    let val = val.trim_matches(char::is_whitespace);
                    match tp {
                        Type::Bool => bcs::to_bytes(&val.parse::<bool>().unwrap()).unwrap(),
                        Type::U8 => bcs::to_bytes(&val.parse::<u8>().unwrap()).unwrap(),
                        Type::U64 => bcs::to_bytes(&val.parse::<u64>().unwrap()).unwrap(),
                        Type::U128 => bcs::to_bytes(&val.parse::<u128>().unwrap()).unwrap(),
                        Type::Address => {
                            bcs::to_bytes(&AccountAddress::from_hex_literal(val).unwrap()).unwrap()
                        }
                        Type::Signer => {
                            bcs::to_bytes(&AccountAddress::from_hex_literal(val).unwrap()).unwrap()
                        }
                        Type::Vector(tp) => match tp.as_ref() {
                            Type::U8 => bcs::to_bytes(&hex::decode(val).unwrap()).unwrap(),
                            _ => unreachable!(),
                        },
                        Type::Reference(tp) => match tp.as_ref() {
                            Type::Signer => {
                                bcs::to_bytes(&AccountAddress::from_hex_literal(val).unwrap())
                                    .unwrap()
                            }
                            _ => unreachable!(),
                        },
                        Type::Struct(_) => U256::from_dec_str(val)
                            .map(|v| bcs::to_bytes(&v.0).unwrap())
                            .unwrap(),
                        _ => unreachable!(),
                    }
                })
                .collect::<Vec<Vec<u8>>>();
            Ok(res)
        } else {
            let signer = bcs::to_bytes(&AccountAddress::from_hex_literal(signer).unwrap())?;
            Ok(vec![signer])
        }
    }

    fn eth_call(&self, ident: &str) -> Result<Option<&ethabi::Function>> {
        let entry = self.entries.functions_by_name(ident)?.first();
        Ok(entry)
    }
}

#[derive(Deserialize)]
pub struct U256Wrapper([u64; 4]);

#[derive(Debug)]
pub struct ExecutionResult {
    pub returns: Vec<Token>,
    pub events: Vec<Event>,
}

impl ExecutionResult {
    pub fn to_result_str(&self) -> String {
        self.returns.iter().map(|val| format!("{val:?}")).join(", ")
    }
}

#[derive(Default, Clone)]
struct Resolver {
    state_data: HashMap<StateKey, Vec<u8>>,
}

impl Resolver {
    pub fn apply(&mut self, output: ChangeSetExt) {
        for (state_key, write_op) in output.write_set() {
            match write_op {
                WriteOp::Modification(blob) | WriteOp::Creation(blob) => {
                    self.set(state_key.clone(), blob.clone());
                }
                WriteOp::Deletion => {
                    self.remove(state_key);
                }
            }
        }
    }

    pub fn set(&mut self, state_key: StateKey, data_blob: Vec<u8>) -> Option<Vec<u8>> {
        self.state_data.insert(state_key, data_blob)
    }

    pub fn remove(&mut self, state_key: &StateKey) -> Option<Vec<u8>> {
        self.state_data.remove(state_key)
    }
}

impl StateView for Resolver {
    fn get_state_value(&self, state_key: &StateKey) -> Result<Option<Vec<u8>>> {
        Ok(self.state_data.get(state_key).cloned())
    }

    fn is_genesis(&self) -> bool {
        self.state_data.is_empty()
    }

    fn get_usage(&self) -> Result<StateStorageUsage> {
        let mut usage = StateStorageUsage::new_untracked();
        for (k, v) in self.state_data.iter() {
            usage.add_item(k.size() + v.len())
        }
        Ok(usage)
    }
}
