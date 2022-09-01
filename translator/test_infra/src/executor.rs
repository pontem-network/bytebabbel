use crate::stdlib::publish_std;
use anyhow::{ensure, Result};
use aptos_aggregator::transaction::ChangeSetExt;
use aptos_crypto::HashValue;
use aptos_gas::{AbstractValueSizeGasParameters, NativeGasParameters};
use aptos_state_view::state_storage_usage::StateStorageUsage;
use aptos_state_view::StateView;
use aptos_types::state_store::state_key::StateKey;
use aptos_types::write_set::WriteOp;
use aptos_vm::data_cache::StorageAdapter;
use aptos_vm::move_vm_ext::{MoveVmExt, SessionExt, SessionId};
use aptos_vm::natives::configure_for_unit_test;
use move_core_types::account_address::AccountAddress;
use move_core_types::effects::Event;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{ModuleId, CORE_CODE_ADDRESS};
use move_core_types::value::MoveTypeLayout;
use move_vm_types::gas::UnmeteredGasMeter;
use move_vm_types::loaded_data::runtime_types::Type;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::str::FromStr;

static INSTANCE: OnceCell<Resolver> = OnceCell::new();

pub struct MoveExecutor {
    resolver: Resolver,
    vm: MoveVmExt,
    seq: u64,
}

impl MoveExecutor {
    pub fn new() -> MoveExecutor {
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
                let output = session.finish().unwrap().into_change_set(&mut ()).unwrap();
                resolver.apply(output);
                resolver
            })
            .clone();

        MoveExecutor {
            resolver,
            vm: Self::create_vm(),
            seq: 1,
        }
    }

    fn create_vm() -> MoveVmExt {
        configure_for_unit_test();
        MoveVmExt::new(
            NativeGasParameters::zeros(),
            AbstractValueSizeGasParameters::zeros(),
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
        let output = session.finish().unwrap().into_change_set(&mut ()).unwrap();
        self.resolver.apply(output);
    }

    pub fn run(&mut self, ident: &str, params: &str) -> Result<ExecutionResult> {
        let (module_id, ident) = Self::prepare_ident(ident);
        let id = SessionId::Txn {
            sender: *module_id.address(),
            sequence_number: self.seq,
            script_hash: HashValue::sha3_256_of(params.as_bytes()).to_vec(),
        };
        self.seq += 1;

        let adapter = StorageAdapter::new(&self.resolver);
        let mut session = self.vm.new_session(&adapter, id);

        let args = Self::prepare_args(params, &module_id, &ident, &session)?;
        let returns = session
            .execute_entry_function(&module_id, &ident, vec![], args, &mut UnmeteredGasMeter)?
            .return_values;
        let result = session.finish().unwrap();
        let events = result.events.clone();
        let output = result.into_change_set(&mut ()).unwrap();
        self.resolver.apply(output);
        Ok(ExecutionResult { returns, events })
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

    fn prepare_args(
        args: &str,
        module_id: &ModuleId,
        ident: &Identifier,
        session: &SessionExt<StorageAdapter<Resolver>>,
    ) -> Result<Vec<Vec<u8>>> {
        let fun = session.load_function(module_id, ident, &[])?;
        let arguments = args.split(',').collect::<Vec<_>>();

        ensure!(
            (arguments.len() == fun.parameters.len())
                || (args.is_empty() && fun.parameters.is_empty())
        );

        arguments
            .into_iter()
            .zip(fun.parameters)
            .map(|(val, tp)| {
                let val = val.trim_matches(char::is_whitespace);
                let bval = match tp {
                    Type::Bool => bcs::to_bytes(&val.parse::<bool>()?),
                    Type::U8 => bcs::to_bytes(&val.parse::<u8>()?),
                    Type::U64 => bcs::to_bytes(&val.parse::<u64>()?),
                    Type::U128 => bcs::to_bytes(&val.parse::<u128>()?),
                    Type::Address | Type::Signer => {
                        bcs::to_bytes(&AccountAddress::from_hex_literal(val)?)
                    }
                    Type::Vector(tp) => match tp.as_ref() {
                        Type::U8 => bcs::to_bytes(&hex::decode(val)?),
                        _ => unreachable!(),
                    },
                    Type::Reference(tp) => match tp.as_ref() {
                        Type::Signer => bcs::to_bytes(&AccountAddress::from_hex_literal(val)?),
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }?;
                Ok(bval)
            })
            .collect::<Result<Vec<Vec<u8>>>>()
    }
}

impl Default for MoveExecutor {
    fn default() -> Self {
        MoveExecutor::new()
    }
}

#[derive(Debug)]
pub struct ExecutionResult {
    pub returns: Vec<(Vec<u8>, MoveTypeLayout)>,
    pub events: Vec<Event>,
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
