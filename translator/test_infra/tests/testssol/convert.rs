use ethabi::Token;
use itertools::Itertools;
use move_core_types::value::MoveValue;

pub trait ResultToString {
    fn to_result_str(&self) -> String;
}

impl ResultToString for MoveValue {
    fn to_result_str(&self) -> String {
        self.to_string()
    }
}

impl ResultToString for Token {
    fn to_result_str(&self) -> String {
        format!("{self:?}")
    }
}

impl<T> ResultToString for Vec<T>
where
    T: ResultToString,
{
    fn to_result_str(&self) -> String {
        format!(
            "({})",
            self.iter().map(|item| item.to_result_str()).join(", ")
        )
    }
}
