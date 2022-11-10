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
                    .split_once(':')
                    .map_or(("", param.trim()), |(tp, value)| (tp.trim(), value.trim()))
            })
            .map(|(tp, value)| (tp.to_string(), value.to_string()))
            .collect::<Vec<(String, String)>>();

        // If there is an alias address(profile name) in the arguments
        let profiles = move_executor::profile::load_configs()
            .unwrap_or_default()
            .profiles
            .unwrap_or_default();

        for (.., value) in args.iter_mut() {
            if let Some(address) = profiles.get(value).and_then(|profile| profile.account) {
                *value = address.to_hex_literal();
            }
        }

        FunctionArgs(args)
    }
}

/// profile name + list args
///
impl From<(&str, &Vec<String>)> for FunctionArgs {
    fn from(from_value: (&str, &Vec<String>)) -> Self {
        let (self_profile_name, value) = from_value;
        let mut args = FunctionArgs::from(value);

        if !args.0.iter().any(|(.., value)| value == "self") {
            return args;
        }

        let profile_address_hex = match move_executor::profile::load_profile(self_profile_name)
            .ok()
            .and_then(|profile| profile.account)
        {
            Some(account) => account.to_hex_literal(),
            None => return args,
        };

        for (.., value) in args.0.iter_mut() {
            if value == "self" {
                *value = profile_address_hex.clone()
            }
        }

        args
    }
}
