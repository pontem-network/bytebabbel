use crate::abi::inc_ret_param::types::ParamType;
use serde::Deserialize;

pub mod types;

/// Аn incoming or returned parameter in a function
#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct Param {
    /// Variable name
    pub name: String,

    // uint, uint256, int256, bytes2 ... or custom
    #[serde(rename = "type")]
    pub tp: ParamType,

    // used for tuple types (more below).
    pub components: Option<String>,

    // if the field is part of the log’s topics, false if it one of the log’s data segment.
    pub indexed: Option<bool>,
}

impl Param {
    pub fn size(&self) -> usize {
        32
    }
}

#[cfg(test)]
mod test {
    use crate::abi::inc_ret_param::Param;

    #[test]
    fn test_deserialize_for_function() {
        // input or return type of a parameter in a function
        let cont = r#"[{ "internalType": "address", "name": "a", "type": "address" }, { "internalType": "address", "name": "", "type": "address" }, { "internalType": "bool[]", "name": "a1", "type": "bool[]" }, { "internalType": "bool", "name": "a2", "type": "bool" }, { "internalType": "bool[3]", "name": "a3", "type": "bool[3]" }, { "internalType": "bool", "name": "", "type": "bool" }, { "internalType": "bytes", "name": "a", "type": "bytes" }, { "internalType": "bytes1", "name": "a1", "type": "bytes1" }, { "internalType": "bytes2", "name": "a2", "type": "bytes2" }, { "internalType": "bytes3", "name": "a3", "type": "bytes3" }, { "internalType": "bytes4", "name": "a4", "type": "bytes4" }, { "internalType": "bytes5", "name": "a5", "type": "bytes5" }, { "internalType": "bytes6", "name": "a6", "type": "bytes6" }, { "internalType": "bytes7", "name": "a7", "type": "bytes7" }, { "internalType": "bytes8", "name": "a8", "type": "bytes8" }, { "internalType": "bytes1", "name": "", "type": "bytes1" }, { "internalType": "contract CusTypes", "name": "a", "type": "address" }, { "internalType": "contract MyTypes", "name": "b", "type": "address" }, { "internalType": "contract CusTypes", "name": "", "type": "address" }, { "internalType": "int256", "name": "a1", "type": "int256" }, { "internalType": "int8", "name": "a2", "type": "int8" }, { "internalType": "int16", "name": "a3", "type": "int16" }, { "internalType": "int16", "name": "a5", "type": "int16" }, { "internalType": "int32", "name": "a6", "type": "int32" }, { "internalType": "int64", "name": "a7", "type": "int64" }, { "internalType": "int128", "name": "a8", "type": "int128" }, { "internalType": "int256", "name": "a9", "type": "int256" }, { "internalType": "int256", "name": "", "type": "int256" }, { "internalType": "int256[]", "name": "a1", "type": "int256[]" }, { "internalType": "int256[2]", "name": "a2", "type": "int256[2]" }, { "internalType": "int8[]", "name": "a3", "type": "int8[]" }, { "internalType": "int256[3]", "name": "", "type": "int256[3]" }, { "internalType": "string", "name": "a", "type": "string" }, { "internalType": "string", "name": "", "type": "string" }, { "internalType": "uint256", "name": "a1", "type": "uint256" }, { "internalType": "uint8", "name": "a2", "type": "uint8" }, { "internalType": "uint16", "name": "a3", "type": "uint16" }, { "internalType": "uint16", "name": "a5", "type": "uint16" }, { "internalType": "uint32", "name": "a6", "type": "uint32" }, { "internalType": "uint64", "name": "a7", "type": "uint64" }, { "internalType": "uint128", "name": "a8", "type": "uint128" }, { "internalType": "uint256", "name": "a9", "type": "uint256" }, { "internalType": "uint256", "name": "", "type": "uint256" }, { "internalType": "uint256[]", "name": "a1", "type": "uint256[]" }, { "internalType": "uint8[200]", "name": "a2", "type": "uint8[200]" }, { "internalType": "uint256[3]", "name": "", "type": "uint256[3]" } ]"#;
        let _: Vec<Param> = serde_json::from_str(cont).unwrap();
    }
}
