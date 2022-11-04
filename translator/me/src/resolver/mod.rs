use std::collections::HashMap;

use anyhow::Result;
use reqwest::Url;

use aptos_aggregator::transaction::ChangeSetExt;
use aptos_state_view::StateView;
use aptos_types::state_store::{state_key::StateKey, state_storage_usage::StateStorageUsage};
use aptos_types::write_set::WriteOp;

use crate::load::load_table_handle_u256;
use move_core_types::account_address::AccountAddress;

pub mod print_access_path;

#[derive(Default, Clone)]
pub struct Resolver {
    pub state_data: HashMap<StateKey, Vec<u8>>,
    pub handler_data: HashMap<AccountAddress, HandleRequest>,
}

impl StateView for Resolver {
    fn get_state_value(&self, state_key: &StateKey) -> Result<Option<Vec<u8>>> {
        let result = self.state_data.get(state_key).cloned();
        if result.is_some() {
            return Ok(result);
        }

        if let StateKey::TableItem { handle, key } = state_key {
            if let Some(data) = self.handler_data.get(&handle.0) {
                return load_table_handle_u256(data, key);
            }
        }

        Ok(None)
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

#[derive(Debug, Clone)]
pub struct HandleRequest {
    pub url: Url,
    pub key_type: String,
    pub value_type: String,
}
