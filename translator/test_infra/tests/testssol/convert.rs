use eth::abi::inc_ret_param::value::ParamValue;
use evm::utils::I256;
use move_core_types::value::MoveValue;
use primitive_types::U256;

pub trait ResultToString {
    fn to_result_str(&self) -> String;
}

impl ResultToString for ParamValue {
    fn to_result_str(&self) -> String {
        match self {
            ParamValue::Bool(value) => value.to_string(),
            ParamValue::Int { value, size: _ } => {
                if value.is_negative() {
                    format!("-{}", U256::from(*value / I256::minus_one()))
                } else {
                    U256::from(*value).to_string()
                }
            }
            ParamValue::UInt { value, size: _ } => format!("{value}"),
            ParamValue::Byte(..) => {
                todo!()
            }
            ParamValue::Bytes(..) => {
                todo!()
            }
            ParamValue::Address(..) => {
                todo!()
            }
            ParamValue::String(value) => String::from_utf8(value.clone()).unwrap_or_default(),
            ParamValue::Array(..) => {
                todo!()
                // if let Some(size) = size {
                //     format!("{tp:?}[{size}]")
                // } else {
                //     format!("{tp:?}[]")
                // }
            }
            ParamValue::Custom { .. } => {
                todo!()
            }
        }
    }
}

impl ResultToString for MoveValue {
    fn to_result_str(&self) -> String {
        self.to_string()
    }
}

impl<T> ResultToString for Vec<T>
where
    T: ResultToString,
{
    fn to_result_str(&self) -> String {
        let list: Vec<String> = self.iter().map(|item| item.to_result_str()).collect();
        format!("{list:?}")
    }
}
