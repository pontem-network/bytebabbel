use std::{
    collections::BTreeMap,
    convert::{From, Into},
    fmt,
    result::Result,
    str::FromStr,
};

use anyhow::format_err;
use serde::{de::Error as _, Deserialize, Deserializer};

use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
    parser::{parse_struct_tag, parse_type_tag},
};

/// A parsed Move resource
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct MoveResource {
    #[serde(rename = "type")]
    pub typ: MoveStructTag,
    pub data: MoveStructValue,
}

/// A JSON map representation of a Move struct's inner types
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct MoveStructValue(pub BTreeMap<Identifier, serde_json::Value>);

/// A Move struct tag for referencing an onchain struct type
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MoveStructTag {
    pub address: AccountAddress,
    pub module: Identifier,
    pub name: Identifier,
    /// Generic type parameters associated with the struct
    pub generic_type_params: Vec<MoveType>,
}

impl FromStr for MoveStructTag {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self, anyhow::Error> {
        Ok(parse_struct_tag(s)?.into())
    }
}

impl From<StructTag> for MoveStructTag {
    fn from(tag: StructTag) -> Self {
        Self {
            address: tag.address,
            module: tag.module,
            name: tag.name,
            generic_type_params: tag.type_params.into_iter().map(MoveType::from).collect(),
        }
    }
}

impl fmt::Display for MoveStructTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::{}::{}", self.address, self.module, self.name)?;
        if let Some(first_ty) = self.generic_type_params.first() {
            write!(f, "<")?;
            write!(f, "{}", first_ty)?;
            for ty in self.generic_type_params.iter().skip(1) {
                write!(f, ", {}", ty)?;
            }
            write!(f, ">")?;
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for MoveStructTag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data = <String>::deserialize(deserializer)?;
        data.parse().map_err(D::Error::custom)
    }
}

/// An enum of Move's possible types on-chain
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MoveType {
    /// A bool type
    Bool,
    /// An 8-bit unsigned int
    U8,
    /// A 64-bit unsigned int
    U64,
    /// A 128-bit unsigned int
    U128,
    /// A 32-byte account address
    Address,
    /// An account signer
    Signer,
    /// A Vector of [`MoveType`]
    Vector { items: Box<MoveType> },
    /// A struct of [`MoveStructTag`]
    Struct(MoveStructTag),
    /// A generic type param with index
    GenericTypeParam { index: u16 },
    /// A reference
    Reference { mutable: bool, to: Box<MoveType> },
    /// A move type that couldn't be parsed
    ///
    /// This prevents the parser from just throwing an error because one field
    /// was unparsable, and gives the value in it.
    Unparsable(String),
}

impl fmt::Display for MoveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoveType::U8 => write!(f, "u8"),
            MoveType::U64 => write!(f, "u64"),
            MoveType::U128 => write!(f, "u128"),
            MoveType::Address => write!(f, "address"),
            MoveType::Signer => write!(f, "signer"),
            MoveType::Bool => write!(f, "bool"),
            MoveType::Vector { items } => write!(f, "vector<{}>", items),
            MoveType::Struct(s) => write!(f, "{}", s),
            MoveType::GenericTypeParam { index } => write!(f, "T{}", index),
            MoveType::Reference { mutable, to } => {
                if *mutable {
                    write!(f, "&mut {}", to)
                } else {
                    write!(f, "&{}", to)
                }
            }
            MoveType::Unparsable(string) => write!(f, "unparsable<{}>", string),
        }
    }
}

// This function cannot handle the full range of types that MoveType can
// represent. Internally, it uses parse_type_tag, which cannot handle references
// or generic type parameters. This function adds nominal support for references
// on top of parse_type_tag, but it still does not work for generic type params.
// For that, we have the Unparsable variant of MoveType, so the deserialization
// doesn't fail when dealing with these values.
impl FromStr for MoveType {
    type Err = anyhow::Error;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let mut is_ref = false;
        let mut is_mut = false;
        if s.starts_with('&') {
            s = &s[1..];
            is_ref = true;
        }
        if is_ref && s.starts_with("mut ") {
            s = &s[4..];
            is_mut = true;
        }
        // Previously this would just crap out, but this meant the API could
        // return a serialized version of an object and not be able to
        // deserialize it using that same object.
        let inner = match parse_type_tag(s) {
            Ok(inner) => inner.into(),
            Err(_e) => MoveType::Unparsable(s.to_string()),
        };
        if is_ref {
            Ok(MoveType::Reference {
                mutable: is_mut,
                to: Box::new(inner),
            })
        } else {
            Ok(inner)
        }
    }
}

// This deserialization has limitations, see the FromStr impl for MoveType.
impl<'de> Deserialize<'de> for MoveType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data = <String>::deserialize(deserializer)
            .map_err(|e| D::Error::custom(format_err!("deserialize Move type failed, {}", e)))?;
        data.parse().map_err(D::Error::custom)
    }
}

impl From<TypeTag> for MoveType {
    fn from(tag: TypeTag) -> Self {
        match tag {
            TypeTag::Bool => MoveType::Bool,
            TypeTag::U8 => MoveType::U8,
            TypeTag::U64 => MoveType::U64,
            TypeTag::U128 => MoveType::U128,
            TypeTag::Address => MoveType::Address,
            TypeTag::Signer => MoveType::Signer,
            TypeTag::Vector(v) => MoveType::Vector {
                items: Box::new(MoveType::from(*v)),
            },
            TypeTag::Struct(v) => MoveType::Struct((*v).into()),
        }
    }
}

/// A Move module Id
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MoveModuleId {
    pub address: AccountAddress,
    pub name: Identifier,
}

impl fmt::Display for MoveModuleId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}::{}", self.address, self.name)
    }
}

impl FromStr for MoveModuleId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((address, name)) = s.split_once("::") {
            return Ok(Self {
                address: address.parse().map_err(|_| invalid_move_module_id(s))?,
                name: name.parse().map_err(|_| invalid_move_module_id(s))?,
            });
        }
        Err(invalid_move_module_id(s))
    }
}

#[inline]
fn invalid_move_module_id<S: fmt::Display + Sized>(s: S) -> anyhow::Error {
    format_err!("Invalid Move module ID: {}", s)
}

impl<'de> Deserialize<'de> for MoveModuleId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let module_id = <String>::deserialize(deserializer)?;
        module_id.parse().map_err(D::Error::custom)
    }
}
