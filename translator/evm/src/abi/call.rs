use anyhow::{anyhow, bail, ensure, Result};

use crate::abi::inc_ret_param::types::ParamType;
use crate::abi::inc_ret_param::value::{AsParamValue, ParamValue};
use crate::abi::Entry;

#[derive(Debug, Clone)]
pub struct CallFn<'a> {
    entry: &'a Entry,
    input: Vec<Option<ParamValue>>,
}

impl<'a> CallFn<'a> {
    fn inputs_types(&'a self) -> Result<Vec<&'a ParamType>> {
        let param = self
            .entry
            .inputs()
            .ok_or(anyhow!("The object is not a function"))?;
        Ok(param.iter().map(|p| &p.tp).collect())
    }

    fn are_all_inputs_filled(&self) -> Result<()> {
        if let Some(position) = self.input.iter().position(|item| item.is_none()) {
            let types = self.inputs_types()?;
            let need_type = types[position];
            bail!("Fill in all incoming parameters for the function. Expected data for {position} param. Type: {need_type:?}")
        }
        return Ok(());
    }
}

impl<'a> CallFn<'a> {
    pub fn set_input<T>(&mut self, number_position: usize, value: T) -> Result<&mut Self>
    where
        T: AsParamValue,
    {
        let types = self.inputs_types()?;
        let tp = types.get(number_position).ok_or(anyhow!(
            "The function parameter number is specified incorrectly"
        ))?;
        let value = tp.set_value(value)?;

        self.input[number_position] = Some(value);

        Ok(self)
    }

    pub fn encode_input(&self) -> Result<String> {
        self.are_all_inputs_filled()?;

        let method_id = self.entry.hash_hex();

        let params: Vec<u8> = self
            .input
            .iter()
            .filter_map(|v| match v {
                // @todo None
                Some(v) => Some(v.encode(None)),
                None => None,
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect();
        let params_hex = hex::encode(&params);
        Ok(format!("0x{method_id}{params_hex}"))
    }
}

pub trait ToCall {
    fn try_call(&self) -> Result<CallFn>;
}

impl ToCall for &Entry {
    fn try_call(&self) -> Result<CallFn> {
        ensure!(self.is_function(), "Is not a function");
        let count = self.inputs().ok_or(anyhow!("Is not a function"))?.len();
        let input = vec![None; count];
        Ok(CallFn { entry: self, input })
    }
}
