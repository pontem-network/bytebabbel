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

    // uint, uint256, int256, bytes2 ... or custom
    #[serde(rename = "internalType")]
    pub internal_type: Option<ParamType>,

    // if the field is part of the log’s topics, false if it one of the log’s data segment.
    pub indexed: Option<bool>,

    // used for tuple types (more below).
    // NOTE: Did not find where it is used. https://docs.soliditylang.org/en/v0.8.15/abi-spec.html#json
    pub components: Option<String>,
}

impl Param {
    pub fn size(&self) -> usize {
        32
    }
}

#[cfg(test)]
mod test {
    use crate::abi::inc_ret_param::types::ParamType;
    use crate::abi::inc_ret_param::Param;

    #[test]
    fn test_deserialize_consturctor() {
        // constructor(bytes32 name_) { }
        let cont = r#"{
            "internalType": "bytes32",
            "name": "name_",
            "type": "bytes32"
        }"#;
        let result: Param = serde_json::from_str(cont).unwrap();
        assert_eq!(
            result,
            Param {
                name: "name_".to_string(),
                tp: ParamType::Byte(32),
                internal_type: Some(ParamType::Byte(32)),
                components: None,
                indexed: None
            }
        );
    }

    #[test]
    fn test_deserialize_event() {
        // event Received(address, uint);
        let mut cont = r#"[
            {
                "indexed": false,
                "internalType": "address",
                "name": "",
                "type": "address"
            },
            {
                "indexed": false,
                "internalType": "uint256",
                "name": "",
                "type": "uint256"
            }
        ]"#;
        let mut result: Vec<Param> = serde_json::from_str(cont).unwrap();
        assert_eq!(
            result[1],
            Param {
                name: String::new(),
                tp: ParamType::Uint(256),
                internal_type: Some(ParamType::Uint(256)),
                indexed: Some(false),
                components: None
            }
        );

        // event Deposit(
        //     address indexed from,
        //     bytes32 indexed id,
        //     uint value
        // );
        cont = r#"[
            {
                "indexed": true,
                "internalType": "address",
                "name": "from",
                "type": "address"
            },
            {
                "indexed": true,
                "internalType": "bytes32",
                "name": "id",
                "type": "bytes32"
            },
            {
                "indexed": false,
                "internalType": "uint256",
                "name": "value",
                "type": "uint256"
            }
        ]"#;
        result = serde_json::from_str(cont).unwrap();
        assert_eq!(
            result[0],
            Param {
                name: "from".to_string(),
                tp: ParamType::Address,
                internal_type: Some(ParamType::Address),
                indexed: Some(true),
                components: None
            }
        );
    }

    #[test]
    fn test_deserialize_function() {
        // function transfer(address newOwner) public { }
        let mut cont = r#"{
                "internalType": "address",
                "name": "newOwner",
                "type": "address"
            }"#;
        let result: Param = serde_json::from_str(cont).unwrap();
        assert_eq!(
            result,
            Param {
                name: "newOwner".to_string(),
                tp: ParamType::Address,
                internal_type: Some(ParamType::Address),
                indexed: None,
                components: None
            }
        );

        // function returnTuple() public pure returns (uint, int, bool) {}
        // Outputs:
        cont = r#"[
            {
                "internalType": "uint256",
                "name": "",
                "type": "uint256"
            },
            {
                "internalType": "int256",
                "name": "",
                "type": "int256"
            },
            {
                "internalType": "bool",
                "name": "",
                "type": "bool"
            }
        ]"#;
        let result: Vec<Param> = serde_json::from_str(cont).unwrap();
        assert_eq!(
            result,
            vec![
                Param {
                    name: String::new(),
                    tp: ParamType::Uint(256),
                    internal_type: Some(ParamType::Uint(256)),
                    indexed: None,
                    components: None
                },
                Param {
                    name: String::new(),
                    tp: ParamType::Int(256),
                    internal_type: Some(ParamType::Int(256)),
                    indexed: None,
                    components: None
                },
                Param {
                    name: String::new(),
                    tp: ParamType::Bool,
                    internal_type: Some(ParamType::Bool),
                    indexed: None,
                    components: None
                },
            ]
        );
    }

    #[test]
    fn test_deserialize_error() {
        // error InsufficientBalance(uint256 available, uint256 required);
        let cont = r#"[
            {
                "internalType": "uint256",
                "name": "available",
                "type": "uint256"
            },
            {
                "internalType": "uint256",
                "name": "required",
                "type": "uint256"
            }
        ]"#;
        let result: Vec<Param> = serde_json::from_str(cont).unwrap();
        assert_eq!(
            result,
            vec![
                Param {
                    name: "available".to_string(),
                    tp: ParamType::Uint(256),
                    internal_type: Some(ParamType::Uint(256)),
                    indexed: None,
                    components: None
                },
                Param {
                    name: "required".to_string(),
                    tp: ParamType::Uint(256),
                    internal_type: Some(ParamType::Uint(256)),
                    indexed: None,
                    components: None
                }
            ]
        );
    }
}
