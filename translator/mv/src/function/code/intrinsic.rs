use move_binary_format::file_format::Bytecode;

pub mod math;

pub fn int_as_bool() -> Vec<Bytecode> {
    vec![Bytecode::LdU128(0), Bytecode::Neq]
}

pub fn is_zero_uint() -> Vec<Bytecode> {
    vec![Bytecode::LdU128(0), Bytecode::Eq]
}

pub fn is_zero_bool() -> Vec<Bytecode> {
    vec![Bytecode::Not]
}
