use anyhow::Result;
use eth::abi::call::to_token;

#[derive(Debug, Clone)]
pub(crate) struct FunctionArgs(Vec<(String, String)>);

impl FunctionArgs {
    pub(crate) fn value(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|(.., val)| val.as_str())
    }

    pub(crate) fn args_encode(&self) -> Result<String> {
        let eth_data = self
            .0
            .iter()
            .map(|(type_str, val_str)| {
                let tp = ethabi::param_type::Reader::read(type_str)?;
                let val_str = val_str.to_string();

                let value = to_token(&(tp, &val_str))?;
                log::trace!("{:?}", &value);

                Ok(value)
            })
            .collect::<Result<Vec<_>>>()?;
        log::trace!("{:?}", &eth_data);

        let result = hex::encode(ethabi::encode(&eth_data));
        log::trace!("{}", &result);

        Ok(result)
    }
}

impl From<&Vec<String>> for FunctionArgs {
    fn from(value: &Vec<String>) -> Self {
        let mut args = value
            .iter()
            .map(|param| {
                param
                    .trim()
                    .split_once(":")
                    .map_or(("", param.trim()), |(tp, value)| (tp.trim(), value.trim()))
            })
            .map(|(tp, value)| (tp.to_string(), value.to_string()))
            .collect::<Vec<(String, String)>>();

        let filter_profile_name = |row: &(String, String)| match row.0.as_str() {
            "hex" | "address" => !row.1.starts_with("0x"),
            _ => false,
        };

        // If there is an alias address(profile name) in the arguments
        if args.iter().any(filter_profile_name) {
            let profiles = move_executor::profile::load_configs()
                .unwrap_or_default()
                .profiles
                .unwrap_or_default();

            for (.., value) in args.iter_mut().filter(|tp| filter_profile_name(tp)) {
                if let Some(address) = profiles.get(value).and_then(|profile| profile.account) {
                    *value = address.to_hex_literal();
                }
            }
        }

        FunctionArgs(args)
    }
}
