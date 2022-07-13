use anyhow::{Error, Result};
use move_core_types::account_address::AccountAddress;
use move_core_types::effects::{ChangeSet, Event};
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{ModuleId, StructTag, CORE_CODE_ADDRESS};
use move_core_types::resolver::{ModuleResolver, ResourceResolver};
use move_core_types::value::MoveTypeLayout;
use move_vm_runtime::move_vm::MoveVM;
use move_vm_runtime::session::Session;
use move_vm_types::gas_schedule::GasStatus;
use move_vm_types::loaded_data::runtime_types::Type;
use std::collections::HashMap;
use std::str::FromStr;

pub struct MoveExecutor {
    resolver: Resolver,
    vm: MoveVM,
}

impl MoveExecutor {
    pub fn new() -> MoveExecutor {
        let natives = move_stdlib::natives::all_natives(CORE_CODE_ADDRESS);
        let vm = MoveVM::new(natives).unwrap();
        MoveExecutor {
            resolver: Default::default(),
            vm,
        }
    }

    pub fn deploy(&mut self, addr: &str, module: Vec<u8>) {
        let mut session = self.vm.new_session(&self.resolver);
        let mut gas_status = GasStatus::new_unmetered();
        session
            .publish_module(
                module,
                AccountAddress::from_hex_literal(addr).unwrap(),
                &mut gas_status,
            )
            .unwrap();
        let (ds, _) = session.finish().unwrap();
        self.resolver.apply(ds);
    }

    pub fn run(&mut self, ident: &str, params: &str) -> Result<ExecutionResult> {
        let (module_id, ident) = Self::prepare_ident(ident);
        let mut session = self.vm.new_session(&self.resolver);
        let mut gas_status = GasStatus::new_unmetered();
        let args = Self::prepare_args(params, &module_id, &ident, &session)?;
        let returns = session
            .execute_entry_function(&module_id, &ident, vec![], args, &mut gas_status)?
            .return_values;
        let (ds, events) = session.finish().unwrap();
        self.resolver.apply(ds);
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
        session: &Session<Resolver>,
    ) -> Result<Vec<Vec<u8>>> {
        let fun = session.load_function(module_id, ident, &[])?;
        args.split(",")
            .zip(fun.parameters)
            .map(|(val, tp)| {
                let val = val.trim_matches(char::is_whitespace);
                let bval = match tp {
                    Type::Bool => bcs::to_bytes(&val.parse::<bool>()?),
                    Type::U8 => bcs::to_bytes(&val.parse::<u8>()?),
                    Type::U64 => bcs::to_bytes(&val.parse::<u64>()?),
                    Type::U128 => bcs::to_bytes(&val.parse::<u128>()?),
                    Type::Address => bcs::to_bytes(&AccountAddress::from_hex_literal(val)?),
                    Type::Signer => bcs::to_bytes(&AccountAddress::from_hex_literal(val)?),
                    Type::Vector(tp) => match tp.as_ref() {
                        Type::U8 => bcs::to_bytes(&hex::decode(val)?),
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

#[derive(Default)]
struct Resolver {
    modules: HashMap<ModuleId, Vec<u8>>,
    resources: HashMap<(AccountAddress, StructTag), Vec<u8>>,
}

impl Resolver {
    pub fn apply(&mut self, ds: ChangeSet) {
        for (acc, name, module) in ds.modules() {
            if let Some(module) = module {
                self.modules
                    .insert(ModuleId::new(acc, name.clone()), module.to_vec());
            } else {
                self.modules.remove(&ModuleId::new(acc, name.clone()));
            }
        }
        for (acc, tag, res) in ds.resources() {
            if let Some(res) = res {
                self.resources.insert((acc, tag.clone()), res.to_vec());
            } else {
                self.resources.remove(&(acc, tag.clone()));
            }
        }
    }
}

impl ModuleResolver for Resolver {
    type Error = Error;

    fn get_module(&self, id: &ModuleId) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(self.modules.get(id).cloned())
    }
}

impl ResourceResolver for Resolver {
    type Error = Error;

    fn get_resource(
        &self,
        address: &AccountAddress,
        typ: &StructTag,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(self.resources.get(&(*address, typ.clone())).cloned())
    }
}
