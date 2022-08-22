use crate::abi::call::encode::{enc_offset, encode_value, ParamTypeSize, ValueEncodeType};
use anyhow::{anyhow, bail, ensure, Result};

use crate::abi::inc_ret_param::types::ParamType;
use crate::abi::inc_ret_param::value::{AsParamValue, ParamValue};
use crate::abi::Entry;

pub mod encode;

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
            .ok_or_else(|| anyhow!("The object is not a function"))?;
        Ok(param.iter().map(|p| &p.tp).collect())
    }

    fn are_all_inputs_filled(&self) -> Result<()> {
        if let Some(position) = self.input.iter().position(|item| item.is_none()) {
            let types = self.inputs_types()?;
            let need_type = types[position];
            bail!("Fill in all incoming parameters for the function. Expected data for {position} param. Type: {need_type:?}")
        }
        Ok(())
    }
}

impl<'a> CallFn<'a> {
    pub fn set_input<T>(&mut self, number_position: usize, value: T) -> Result<&mut Self>
    where
        T: AsParamValue,
    {
        let types = self.inputs_types()?;
        let tp = types
            .get(number_position)
            .ok_or_else(|| anyhow!("The function parameter number is specified incorrectly"))?;
        let value = tp.set_value(value)?;

        self.input[number_position] = Some(value);

        Ok(self)
    }

    pub fn encode(&self) -> Result<String> {
        self.are_all_inputs_filled()?;

        let input_types = self.inputs_types()?;
        let method_id = self.entry.hash_hex();

        if self.input.iter().any(|item| item.is_none()) {
            bail!("Not all parameters were filled in");
        }

        let value = self
            .input
            .iter()
            .zip(input_types)
            .filter_map(|(v, tp)| {
                v.as_ref()
                    .map(|v| encode_value(v, tp, 0).map(|value| (tp, value)))
            })
            .collect::<Result<Vec<_>>>()?;

        let mut result = Vec::new();
        let mut ds: Vec<u8> = Vec::new();
        let start = value
            .iter()
            .map(|(tp, _)| tp.size_bytes().unwrap_or(32))
            .sum::<u32>();

        for (_, v) in &value {
            match v {
                ValueEncodeType::Static(data) => {
                    result.extend(data);
                }
                ValueEncodeType::Dynamic(data) => {
                    let offset = start + ds.len() as u32;
                    result.extend(enc_offset(offset));
                    ds.extend(data);
                }
            }
        }
        result.append(&mut ds);

        let params_hex = hex::encode(&result);
        Ok(format!("0x{method_id}{params_hex}"))
    }
}

pub trait ToCall {
    fn try_call(&self) -> Result<CallFn>;
}

impl ToCall for &Entry {
    fn try_call(&self) -> Result<CallFn> {
        ensure!(self.is_function(), "Is not a function");
        let count = self
            .inputs()
            .ok_or_else(|| anyhow!("Is not a function"))?
            .len();
        let input: Vec<Option<ParamValue>> = vec![None; count];
        Ok(CallFn { entry: self, input })
    }
}
