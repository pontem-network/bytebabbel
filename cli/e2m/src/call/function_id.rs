use crate::profile::ProfileValue;
use anyhow::{anyhow, Error, Result};
use std::str::FromStr;

const FUNCTION_ID_EXPECTED_FORMAT: &str = "<ADDRESS>::<MODULE_ID>::<FUNCTION_NAME>";

/// Function name as `<ADDRESS>::<MODULE_ID>::<FUNCTION_NAME>`
///
/// Example:
/// `0x842ed41fad9640a2ad08fdd7d3e4f7f505319aac7d67e1c0dd6a7cce8732c7e3::message::set_message`
#[derive(Debug, Clone)]
pub(crate) struct FunctionId {
    pub address: String,
    pub module: String,
    pub function: String,
}

impl FunctionId {
    /// The profile name will be replaced with the address from `back.aptos/config.yaml`
    fn replace_address_name_to_address(&mut self) -> Result<()> {
        if self.address.starts_with("0x") {
            return Ok(());
        }

        self.address = ProfileValue::from_str(&self.address)?
            .to_address()?
            .to_hex_literal();

        Ok(())
    }
}

impl FromStr for FunctionId {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let fn_err = || anyhow!("{FUNCTION_ID_EXPECTED_FORMAT}");

        let mut parts = s.split("::");

        let address = parts.next().ok_or_else(fn_err)?.to_string();
        let module = parts.next().ok_or_else(fn_err)?.to_string();
        let function = parts.next().ok_or_else(fn_err)?.to_string();

        let mut fnid = FunctionId {
            address,
            module,
            function,
        };
        fnid.replace_address_name_to_address()?;

        Ok(fnid)
    }
}

impl ToString for FunctionId {
    fn to_string(&self) -> String {
        format!("{}::{}::{}", self.address, self.module, self.function)
    }
}
