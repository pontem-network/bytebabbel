use anyhow::{anyhow, ensure, Result};

use crate::abi::inc_ret_param::value::{AsParamValue, ParamValue};
use crate::abi::Entry;

#[derive(Debug)]
pub struct CallFn<'a> {
    entry: &'a Entry,
    input: Vec<Option<ParamValue>>,
}

impl<'a> CallFn<'a> {
    pub fn set_param_by_pos<T>(&mut self, number_position: usize, value: T) -> Result<&mut Self>
    where
        T: AsParamValue,
    {
        let inputs = self
            .entry
            .inputs()
            .ok_or(anyhow!("The object is not a function"))?;
        let tp = &inputs
            .get(number_position)
            .ok_or(anyhow!(
                "The function parameter number is specified incorrectly"
            ))?
            .tp;
        let value = tp.set_value(value)?;
        self.input[number_position] = Some(value);
        Ok(self)
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
