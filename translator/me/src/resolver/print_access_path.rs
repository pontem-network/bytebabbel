use aptos_types::{
    access_path::{AccessPath, Path},
    state_store::state_key::StateKey,
};

pub trait AccessPathToString {
    fn to_string(&self) -> String;
}

impl AccessPathToString for StateKey {
    fn to_string(&self) -> String {
        match self {
            StateKey::AccessPath(acc) => AccessPathToString::to_string(acc),
            StateKey::TableItem { key, handle } => {
                format!(
                    "TableHandle: {}::{}",
                    handle.0.to_hex_literal(),
                    hex::encode(key)
                )
            }
            StateKey::Raw(data) => {
                format!("Raw: {}", hex::encode(data))
            }
        }
    }
}

impl AccessPathToString for AccessPath {
    fn to_string(&self) -> String {
        let path: aptos_types::access_path::Path = bcs::from_bytes(&self.path).unwrap();
        match path {
            Path::Code(code) => format!("Code {}", code),
            Path::Resource(resource) => format!("Resource {}", resource),
        }
    }
}
