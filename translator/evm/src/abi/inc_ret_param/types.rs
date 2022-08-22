use std::fmt::{Debug, Formatter};

use anyhow::{anyhow, bail, Error, Result};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserializer;

use crate::abi::call::encode::ParamTypeSize;

lazy_static! {
    static ref REG_UINT: Regex = Regex::new(r#"^(?P<tp>(u)?int)(?P<size>\d+)?$"#).unwrap();
    static ref REG_BYTES: Regex = Regex::new(r#"^bytes(?P<size>\d+)$"#).unwrap();
    static ref REG_ARRAY: Regex =
        Regex::new(r#"^(?P<next>(?i)[a-z\-_\d]+(\[\d*\])*)(?P<cur>\[\d*\])$"#).unwrap();
}

/// The input or return type of a parameter in a function
#[derive(Eq, PartialEq, Clone)]
pub enum ParamType {
    Bool,
    // 2^3...2^8 = 8 ... 256
    // default: 256
    Int(u16),
    // 2^3...2^8 = 8 ... 256
    // default: 256
    UInt(u16),
    // 1 ... 32
    Byte(u8),
    // Holds a 20 byte value (size of an Ethereum address).
    Address,
    // Dynamically-sized byte array
    Bytes,
    // Dynamically-sized byte array
    String,
    Array {
        tp: Box<ParamType>,
        size: Option<u32>,
    },
    // Not a Primitive type
    Custom(String),
}

impl ParamType {
    pub fn is_static_size(&self) -> bool {
        match self {
            ParamType::Bool => true,
            ParamType::Int(..) => true,
            ParamType::UInt(..) => true,
            ParamType::Byte(..) => true,
            ParamType::Bytes => false,
            ParamType::Address => true,
            ParamType::String => false,
            ParamType::Array { tp, size } => {
                if size.is_some() {
                    tp.is_static_size()
                } else {
                    false
                }
            }
            ParamType::Custom(name) => todo!(),
        }
    }
}

impl ToString for ParamType {
    fn to_string(&self) -> String {
        match self {
            ParamType::Bool => "bool".to_string(),
            ParamType::Int(size) => format!("int{size}"),
            ParamType::UInt(size) => format!("uint{size}"),
            ParamType::Byte(size) => format!("bytes{size}"),
            ParamType::Bytes => "bytes".to_string(),
            ParamType::Address => "address".to_string(),
            ParamType::String => "string".to_string(),
            ParamType::Array { tp, size } => {
                if let Some(size) = size {
                    format!("{tp:?}[{size}]")
                } else {
                    format!("{tp:?}[]")
                }
            }
            ParamType::Custom(name) => format!("{name}"),
        }
    }
}

impl Debug for ParamType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl TryFrom<&str> for ParamType {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let result = match value {
            s if REG_ARRAY.is_match(s) => {
                let capts = REG_ARRAY
                    .captures(s)
                    .ok_or_else(|| anyhow!("incorrect format: {value}"))?;
                let next = capts["next"].trim();
                let cur = capts["cur"]
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .trim();

                let size = if cur.is_empty() {
                    None
                } else {
                    let s = cur
                        .parse()
                        .map_err(|err| anyhow!("incorrect format: {value}\n{err:?}"))?;

                    Some(s)
                };

                let next = ParamType::try_from(next)
                    .map_err(|err| anyhow!("incorrect format: {value}\n{err:?}"))?;
                ParamType::Array {
                    tp: Box::new(next),
                    size,
                }
            }
            "bool" => ParamType::Bool,
            "address" => ParamType::Address,
            "bytes" => ParamType::Bytes,
            "string" => ParamType::String,
            s if REG_UINT.is_match(s) => {
                let caps = REG_UINT
                    .captures(s)
                    .ok_or_else(|| anyhow!("incorrect format: {value}"))?;
                let tp = &caps["tp"];

                let size = caps
                    .name("size")
                    .map(|t| {
                        t.as_str()
                            .parse()
                            .map_err(|err| anyhow!("incorrect format: {value}. {err:?}"))
                    })
                    .unwrap_or(Ok(256))?;

                if !(8..=256).contains(&size) {
                    bail!("Unknown type {value}");
                }
                // 8,16,32..256
                if !(3..=8).any(|p| 2u16.pow(p) == size) {
                    bail!("Unknown type {value}");
                }

                match tp {
                    "int" => ParamType::Int(size),
                    "uint" => ParamType::UInt(size),
                    _ => bail!("incorrect format: {value}"),
                }
            }
            s if REG_BYTES.is_match(s) => {
                let capts = REG_BYTES
                    .captures(s)
                    .ok_or_else(|| anyhow!("incorrect format: {value}"))?;

                let size_str = &capts["size"];
                let size: u8 = size_str
                    .parse()
                    .map_err(|err| anyhow!("Expected number {value}. {err:?}"))?;
                if !(1..=32).contains(&size) {
                    bail!("A number from 1 to 32 was expected. Value: {value}")
                }
                ParamType::Byte(size)
            }
            _ => ParamType::Custom(value.to_string()),
        };
        Ok(result)
    }
}

impl<'de> serde::de::Deserialize<'de> for ParamType {
    fn deserialize<D>(des: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl ::serde::de::Visitor<'_> for Visitor {
            type Value = ParamType;

            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(formatter, "a string for {}", stringify!(ParamType))
            }

            fn visit_str<E>(self, value: &str) -> Result<ParamType, E>
            where
                E: ::serde::de::Error,
            {
                ParamType::try_from(value).map_err(|err| {
                    E::invalid_value(
                        ::serde::de::Unexpected::Other(&format!(
                            "unknown {} variant: {}\n{err:?}",
                            stringify!(ParamType),
                            value
                        )),
                        &self,
                    )
                })
            }
        }

        // Deserialize the enum from a string.
        des.deserialize_str(Visitor)
    }
}

#[cfg(test)]
mod test {
    use crate::abi::inc_ret_param::types::ParamType;
    use itertools::Itertools;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref EXEPLES: Vec<(&'static str, ParamType)> = vec![
            ("bool", ParamType::Bool),
            ("int", ParamType::Int(256)),
            ("uint", ParamType::UInt(256)),
            ("address", ParamType::Address),
            ("bytes", ParamType::Bytes),
            ("string", ParamType::String),
            ("CusType", ParamType::Custom("CusType".to_string())),
            ("MyType", ParamType::Custom("MyType".to_string())),
            ("Int", ParamType::Custom("Int".to_string())),
            ("intfix64", ParamType::Custom("intfix64".to_string())),
            ("int64fix", ParamType::Custom("int64fix".to_string())),
            ("_int64", ParamType::Custom("_int64".to_string())),
            ("bytes1fix", ParamType::Custom("bytes1fix".to_string())),
            (
                "bool[]",
                ParamType::Array {
                    tp: Box::new(ParamType::Bool),
                    size: None,
                },
            ),
            (
                "address[222]",
                ParamType::Array {
                    tp: Box::new(ParamType::Address),
                    size: Some(222),
                },
            ),
            (
                "uint[][]",
                ParamType::Array {
                    tp: Box::new(ParamType::Array {
                        tp: Box::new(ParamType::UInt(256)),
                        size: None,
                    }),
                    size: None,
                },
            ),
            (
                "bytes1[2][][3][]",
                ParamType::Array {
                    tp: Box::new(ParamType::Array {
                        tp: Box::new(ParamType::Array {
                            tp: Box::new(ParamType::Array {
                                tp: Box::new(ParamType::Byte(1)),
                                size: Some(2),
                            }),
                            size: None,
                        }),
                        size: Some(3),
                    }),
                    size: None,
                },
            ),
            (
                "int[3]",
                ParamType::Array {
                    tp: Box::new(ParamType::Int(256)),
                    size: Some(3),
                },
            ),
            (
                "uint[][]",
                ParamType::Array {
                    tp: Box::new(ParamType::Array {
                        tp: Box::new(ParamType::UInt(256)),
                        size: None,
                    }),
                    size: None,
                },
            ),
            (
                "MyType[]",
                ParamType::Array {
                    tp: Box::new(ParamType::Custom("MyType".to_string())),
                    size: None,
                },
            ),
        ];
    }

    #[test]
    fn test_tryfrom() {
        // uint{N}, int{N}
        // 2^3...2^8 = 8 ... 256
        for p in 3..=8 {
            let size = 2u16.pow(p);

            let cont_int = format!("int{size}");
            assert_eq!(
                ParamType::Int(size),
                ParamType::try_from(cont_int.as_str()).unwrap()
            );
            let cont_uint = format!("uint{size}");
            assert_eq!(
                ParamType::UInt(size),
                ParamType::try_from(cont_uint.as_str()).unwrap()
            );
        }
        for size in [0, 3, 4, 24, 255] {
            let r = ParamType::try_from(format!("int{size}").as_str());
            assert!(r.is_err());
            let r = ParamType::try_from(format!("uint{size}").as_str());
            assert!(r.is_err());
        }

        // bytes{N}
        for size in 1..=32 {
            let cont = format!("bytes{size}");
            assert_eq!(
                ParamType::Byte(size),
                ParamType::try_from(cont.as_str()).unwrap()
            );
        }
        for size in [0, 33, 64, 256] {
            let r = ParamType::try_from(format!("bytes{size}").as_str());
            assert!(r.is_err());
        }

        for (cont, value) in EXEPLES.iter() {
            let result = ParamType::try_from(*cont).unwrap();
            assert_eq!(&result, value);
        }
    }

    #[test]
    fn test_deserialize() {
        let cont = format!(
            "[{}]",
            EXEPLES.iter().map(|(s, _)| format!(r#""{s}""#)).join(", ")
        );
        let parse_result: Vec<ParamType> = serde_json::from_str(&cont).unwrap();
        let expected_result: Vec<ParamType> = EXEPLES.iter().map(|(_, r)| r).cloned().collect();

        assert_eq!(parse_result, expected_result);
    }

    #[test]
    fn test_is_static_size() {
        assert!(ParamType::Bool.is_static_size());
        assert!(ParamType::Int(8).is_static_size());
        assert!(ParamType::UInt(8).is_static_size());
        assert!(ParamType::Byte(3).is_static_size());
        assert!(ParamType::Address.is_static_size());
        assert!(!ParamType::Bytes.is_static_size());
        assert!(!ParamType::String.is_static_size());

        assert!(!ParamType::Array {
            size: None,
            tp: Box::new(ParamType::UInt(8))
        }
        .is_static_size());

        assert!(ParamType::Array {
            size: Some(3),
            tp: Box::new(ParamType::UInt(8))
        }
        .is_static_size());

        assert!(ParamType::Array {
            size: Some(3),
            tp: Box::new(ParamType::Array {
                size: Some(3),
                tp: Box::new(ParamType::Bool),
            }),
        }
        .is_static_size());

        assert!(!ParamType::Array {
            size: Some(3),
            tp: Box::new(ParamType::Array {
                size: None,
                tp: Box::new(ParamType::Bool),
            }),
        }
        .is_static_size());

        assert!(!ParamType::Array {
            size: None,
            tp: Box::new(ParamType::Array {
                size: Some(3),
                tp: Box::new(ParamType::Bool),
            }),
        }
        .is_static_size());
    }
}
