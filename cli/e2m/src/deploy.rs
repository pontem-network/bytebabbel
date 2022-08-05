use std::collections::HashMap;
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};
use clap::{App, Parser};

use crate::Args;
use aptos_types::transaction::{ModuleBundle, TransactionPayload};

impl Args {
    pub fn publish(&self, path: &Path) -> Result<String> {
        use clap::{CommandFactory, FromArgMatches, Parser};

        if self.profile_or_address.is_address() {
            bail!(
                "For deploy, you need to specify the profile name. \n\n\
                    Example: \n\
                    $ e2m <path/to/file.sol> --profile <NameProfile>\n\n\
                    Create profile default: \n\
                    $ aptos init\n\n\
                    Create profile with name:\n\
                    $ aptos init --profile <NameProfile>"
            )
        }
        todo!();

        let compiled_units = vec![fs::read(&path)?];
        let transaction = TransactionPayload::ModuleBundle(ModuleBundle::new(compiled_units));
        let txt_option: aptos::common::types::TransactionOptions =
            aptos::common::types::TransactionOptions::try_parse_from(&[
                "subcommand",
                "--profile",
                "demo",
            ])
            .map_err(|err| anyhow!("Invalid profile parameter"))?;
        let fut = txt_option.submit_transaction(transaction);
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
