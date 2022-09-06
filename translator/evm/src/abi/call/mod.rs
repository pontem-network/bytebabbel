use crate::abi::call::encode::{
    decode_value, enc_offset, encode_value, ParamTypeSize, ValueEncodeType,
};
use crate::abi::entries::Entry;
use anyhow::{anyhow, bail, ensure, Result};

use crate::abi::inc_ret_param::types::ParamType;
use crate::abi::inc_ret_param::value::{AsParamValue, ParamValue};

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

    fn outputs_types(&'a self) -> Result<Vec<&'a ParamType>> {
        let param = self
            .entry
            .outputs()
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

    pub fn encode(&self) -> Result<Vec<u8>> {
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

        let mut params = Vec::new();
        let mut ds: Vec<u8> = Vec::new();
        let start = value
            .iter()
            .map(|(tp, _)| tp.size_bytes().unwrap_or(32))
            .sum::<u32>();

        for (_, v) in &value {
            match v {
                ValueEncodeType::Static(data) => {
                    params.extend(data);
                }
                ValueEncodeType::Dynamic(data) => {
                    let offset = start + ds.len() as u32;
                    params.extend(enc_offset(offset));
                    ds.extend(data);
                }
            }
        }
        params.append(&mut ds);

        let mut result = hex::decode(method_id)?;
        result.append(&mut params);
        Ok(result)
    }

    pub fn decode_return(&self, value: Vec<u8>) -> Result<Vec<ParamValue>> {
        let output_types = self.outputs_types()?;
        if output_types.is_empty() {
            return Ok(Vec::new());
        }

        let mut start: usize = 0;
        output_types
            .iter()
            .map(|tp| {
                if let Some(size) = tp.size_bytes() {
                    let size = size as usize;
                    let result = decode_value(&value[start..start + size], tp, 0)?;
                    start += size;
                    return Ok(result);
                }

                let offset = encode::to_usize(&value[start..start + 32]);
                let result = decode_value(&value[offset..], tp, 0)?;
                start += 32;
                Ok(result)
            })
            .collect::<Result<_>>()
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
