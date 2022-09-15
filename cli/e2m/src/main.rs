use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use test_infra::init_log;

pub mod convert;
#[cfg(feature = "deploy")]
pub mod deploy;
pub mod flags;
pub mod profile;

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Args {
    /// Path to the file. Specify the path to sol file or abi|bin.
    #[clap(value_parser)]
    path: PathBuf,

    /// Where to save the converted move binary code
    #[clap(short, long = "output", display_order = 3, value_parser)]
    output_path: Option<PathBuf>,

    /// The name of the Move module. If not specified, the name will be taken from the abi path
    #[clap(long = "module", display_order = 4, value_parser)]
    move_module_name: Option<String>,

    /// Profile name or address. The address must start with "0x". Needed for the module address
    #[clap(
        long = "profile",
        display_order = 5,
        short = 'p',
        default_value = "default",
        value_parser
    )]
    profile_or_address: profile::ProfileValue,

    /// Parameters for initialization
    #[clap(long = "args", short = 'a', default_value = "")]
    init_args: String,

    #[clap(flatten)]
    deploy_flags: flags::DeployFlags,
    #[clap(flatten)]
    translation_flags: flags::TranslationFlags,
}

impl Args {
    pub fn execute(&self) -> Result<String> {
        let result = self.convert()?;

        #[cfg(feature = "deploy")]
        if self.deploy_flags.deploy {
            return self.publish(&result);
        }

        self.translation_flags.check()?;

        Ok(format!(
            "{}\n{}",
            result.mv_path.to_string_lossy(),
            result.move_path.to_string_lossy()
        ))
    }
}

fn main() {
    init_log();

    match Args::parse().execute() {
        Ok(result) => {
            println!("{result}");
        }
        Err(err) => {
            println!("Error: {err:?}");
        }
    }
}
