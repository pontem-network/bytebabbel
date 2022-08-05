use std::fs;
use std::future::Future;
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::Args;
use aptos_types::transaction::{ModuleBundle, TransactionPayload};

impl Args {
    /// Publish in aptos node
    /// Access keys are taken from profiles (.aptos/config.yaml).
    pub fn publish(&self, mv_path: &Path) -> Result<String> {
        use clap::Parser;

        let profile = self.profile_or_address.name_profile().map_err(|_| {
            anyhow!(
                "For deploy, you need to specify the profile name. \n\n\
                    Example: \n\
                    $ e2m <path/to/file.sol> --profile <NameProfile>\n\n\
                    Create profile default: \n\
                    $ aptos init\n\n\
                    Create profile with name:\n\
                    $ aptos init --profile <NameProfile>"
            )
        })?;

        let compiled_units = vec![fs::read(&mv_path)?];
        let transaction = TransactionPayload::ModuleBundle(ModuleBundle::new(compiled_units));
        let txt_option: aptos::common::types::TransactionOptions =
            aptos::common::types::TransactionOptions::try_parse_from(&[
                "subcommand",
                "--profile",
                profile,
            ])
            .map_err(|_| anyhow!("Invalid profile parameter. "))?;
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
