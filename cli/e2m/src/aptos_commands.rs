use std::fs;
use std::future::Future;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};

use aptos_types::transaction::{ModuleBundle, TransactionPayload};

/// CLI tool for interacting with the Aptos blockchain and nodes
#[derive(Subcommand)]
pub enum Tools {
    #[clap(subcommand)]
    Account(aptos::account::AccountTool),

    Init(aptos::common::init::InitTool),

    #[clap(subcommand)]
    Config(aptos::config::ConfigTool),

    #[clap(subcommand)]
    Key(aptos::op::key::KeyTool),

    Publish(PublishPackage),
}

impl Tools {
    pub fn execute(self) -> Result<String> {
        match self {
            Tools::Account(data) => wait(data.execute()).map_err(|err| anyhow!("{err}")),
            Tools::Init(data) => Tools::Config(aptos::config::ConfigTool::Init(data)).execute(),
            Tools::Config(data) => wait(data.execute()).map_err(|err| anyhow!("{err}")),
            Tools::Key(data) => wait(data.execute()).map_err(|err| anyhow!("{err}")),
            Tools::Publish(data) => data.execute(),
        }
    }
}

/// Publishes the modules
#[derive(Debug, Parser)]
pub struct PublishPackage {
    /// Path to the compiled move module
    #[clap(value_parser)]
    path: PathBuf,

    #[clap(flatten)]
    txn_options: aptos::common::types::TransactionOptions,
}

impl PublishPackage {
    pub fn execute(self) -> Result<String> {
        let PublishPackage { path, txn_options } = self;
        let compiled_units = vec![fs::read(&path)?];

        // Send the compiled module using a module bundle
        let fut = txn_options.submit_transaction(TransactionPayload::ModuleBundle(
            ModuleBundle::new(compiled_units),
        ));

        let result = wait(fut).map(aptos::common::types::TransactionSummary::from)?;

        Ok(serde_json::to_string_pretty(&serde_json::to_value(
            &result,
        )?)?)
    }
}

fn wait<F: Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}
