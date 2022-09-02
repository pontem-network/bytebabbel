use evm::abi::inc_ret_param::value::ParamValue;
use move_core_types::value::MoveValue;

pub trait ResultToString {
    fn to_result_str(&self) -> String;
}

impl ResultToString for ParamValue {
    fn to_result_str(&self) -> String {
        match self {
            ParamValue::Bool(value) => value.to_string(),
            ParamValue::Int { value, size } => format!("{value}i{size}"),
            ParamValue::UInt { value, size } => format!("{value}u{size}"),
            ParamValue::Byte(value) => {
                todo!()
            }
            ParamValue::Bytes(value) => {
                todo!()
            }
            ParamValue::Address(value) => {
                todo!()
            }
            ParamValue::String(value) => "string".to_string(),
            ParamValue::Array(value) => {
                todo!()
                // if let Some(size) = size {
                //     format!("{tp:?}[{size}]")
                // } else {
                //     format!("{tp:?}[]")
                // }
            }
            ParamValue::Custom { name, params } => {
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
