use std::fs;

use anyhow::{anyhow, Result};
use move_binary_format::access::ModuleAccess;
use move_binary_format::file_format::{ConstantPoolIndex, SignatureToken};
use move_binary_format::CompiledModule;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::IdentStr;

use crate::Paths;

const TEMPLATE_USE: &str = "\
use crate::Function;
use enum_iterator::Sequence;
use move_binary_format::file_format::{
    ConstantPoolIndex, FunctionHandleIndex, SignatureToken, StructDefinitionIndex,
    StructHandleIndex,
};
";

pub const SELF_ADDRESS: AccountAddress = AccountAddress::new([
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x42,
]);

const SELF_ADDRESS_INDEX: &str = "\
pub fn self_address_index() -> ConstantPoolIndex {
    ConstantPoolIndex(###SELF_ADDRESS_INDEX###)
}
";

const TEMPLATE_ENUM_TOKEN: &str =
    "SignatureToken::Struct(StructHandleIndex(###ENUM_INDEX_HANDLE###))";
const TEMPLATE_ENUM_TOKEN_MUT: &str =
    "SignatureToken::MutableReference(Box::new(SignatureToken::Struct(StructHandleIndex(###ENUM_INDEX_HANDLE###))))";

const TEMPLATE_ENUM: &str = "\
#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence)]
pub enum ###ENUM_NAME### {
    ###ENUM_BODY###
}

impl Function for ###ENUM_NAME### {
    fn name(&self) -> &'static str {
        match self {
            ###ENUM_FN_NAME###
        }
    }

    fn handler(&self) -> FunctionHandleIndex {
        match self {
            ###ENUM_FN_HANDLER###
        }
    }
}
";

const TEMPLATE_ENUM_STRUCTURE: &str = "\
impl ###ENUM_NAME### {
    pub fn token() -> SignatureToken {
        ###ENUM_INDEX_TOKEN###
    }
    pub fn instance() -> StructDefinitionIndex {
        StructDefinitionIndex(###ENUM_INDEX_DEFINITION###)
    }
}
";

const MEMORY_TABLE: [(&str, &str); 9] = [
    ("New", "new_mem"),
    ("Size", "effective_len"),
    ("Load", "mload"),
    ("Store", "mstore"),
    ("Store8", "mstore8"),
    ("Hash", "hash"),
    ("Slice", "mslice"),
    ("RequestBufferLen", "request_buffer_len"),
    ("ReadRequestBuffer", "read_request_buffer"),
];

const PERSIST_TABLE: [(&str, &str); 8] = [
    ("InitContract", "init_contract"),
    ("Store", "sstore"),
    ("Load", "sload"),
    ("Log0", "log0"),
    ("Log1", "log1"),
    ("Log2", "log2"),
    ("Log3", "log3"),
    ("Log4", "log4"),
];

const U256_TABLE: [(&str, &str); 37] = [
    ("Add", "overflowing_add"),
    ("Sub", "overflowing_sub"),
    ("Mul", "overflowing_mul"),
    ("Div", "div"),
    ("Mod", "mod"),
    ("BitOr", "bitor"),
    ("BitAnd", "bitand"),
    ("BitXor", "bitxor"),
    ("Shl", "shl"),
    ("Shr", "shr"),
    ("Lt", "lt"),
    ("Gt", "gt"),
    ("Le", "le"),
    ("Ge", "ge"),
    ("Eq", "eq"),
    ("Neq", "ne"),
    ("BitNot", "bitnot"),
    ("Byte", "byte"),
    ("FromSigner", "from_signer"),
    ("FromBytes", "from_bytes"),
    ("FromBool", "from_bool"),
    ("ToBool", "to_bool"),
    ("FromU64s", "from_u64s"),
    ("IsZero", "is_zero"),
    ("SDiv", "sdiv"),
    ("SLt", "slt"),
    ("SGt", "sgt"),
    ("SMod", "smod"),
    ("Exp", "exp"),
    ("SignExtend", "signextend"),
    ("Sar", "sar"),
    ("FromAddress", "from_address"),
    ("ToAddress", "to_address"),
    ("FromU128", "from_u128"),
    ("ToU128", "as_u128"),
    ("AddMod", "add_mod"),
    ("MulMod", "mul_mod"),
];

const INFO_TABLE: [(&str, &str); 9] = [
    ("AptosBalance", "balance"),
    ("Gas", "gas"),
    ("GasPrice", "gas_price"),
    ("GasLimit", "gas_limit"),
    ("BlockHash", "block_hash"),
    ("BlockHeight", "block_height"),
    ("BlockTimestamp", "block_timestamp"),
    ("BlockDifficulty", "block_difficulty"),
    ("BlockCoinbase", "block_coinbase"),
];

enum EnumType<'a> {
    Structure {
        name: &'a str,
        mut_token: bool,
        table: Vec<(&'a str, &'a str)>,
    },
    Module {
        name: &'a str,
        table: Vec<(&'a str, &'a str)>,
    },
}

impl EnumType<'_> {
    pub fn name(&self) -> &str {
        match self {
            EnumType::Structure { name, .. } => name,
            EnumType::Module { name, .. } => name,
        }
    }

    pub fn talbe_ref(&self) -> &Vec<(&str, &str)> {
        match self {
            EnumType::Structure { table, .. } => table,
            EnumType::Module { table, .. } => table,
        }
    }
}

pub(crate) fn generate_table(paths: &Paths) -> Result<()> {
    let template_mv_cont = fs::read(&paths.template_mv)?;
    let template = CompiledModule::deserialize(&template_mv_cont).unwrap();

    let mut result = [
        EnumType::Structure {
            name: "Memory",
            mut_token: false,
            table: MEMORY_TABLE.to_vec(),
        },
        EnumType::Structure {
            name: "Persist",
            mut_token: true,
            table: PERSIST_TABLE.to_vec(),
        },
        EnumType::Structure {
            name: "U256",
            mut_token: false,
            table: U256_TABLE.to_vec(),
        },
        EnumType::Module {
            name: "Info",
            table: INFO_TABLE.to_vec(),
        },
    ]
    .into_iter()
    .map(|data| gen_enum_code(&template, data))
    .collect::<Result<Vec<String>>>()?
    .join("\n");

    let index = find_address_const(&template, SELF_ADDRESS).ok_or_else(|| {
        anyhow!(
            "Can't find self address index in template module: {:?}",
            paths.template_mv
        )
    })?;
    result += "\n";
    result += &SELF_ADDRESS_INDEX.replace("###SELF_ADDRESS_INDEX###", &index.to_string());

    fs::write(&paths.intrinsic_table, format!("{TEMPLATE_USE}{result}"))?;

    Ok(())
}

fn gen_enum_code(template_mv: &CompiledModule, data: EnumType) -> Result<String> {
    let name = data.name();
    let table = data.talbe_ref();

    let enum_body = table
        .iter()
        .map(|(name, ..)| format!("{name},"))
        .collect::<Vec<String>>()
        .join("\n    ");

    let enum_fn_name = table
        .iter()
        .map(|(name, move_fn)| format!("Self::{name} => \"{move_fn}\","))
        .collect::<Vec<String>>()
        .join("\n            ");

    let enum_fn_handler = table
        .iter()
        .map(|(name, move_fn)| {
            let fn_index = find_function_index(template_mv, move_fn)
                .ok_or_else(|| anyhow!("Function {move_fn:?} not found"))?;
            Ok(format!("Self::{name} => FunctionHandleIndex({fn_index}),"))
        })
        .collect::<Result<Vec<String>>>()?
        .join("\n            ");

    let mut result = TEMPLATE_ENUM
        .replace("###ENUM_NAME###", name)
        .replace("###ENUM_BODY###", &enum_body)
        .replace("###ENUM_FN_NAME###", &enum_fn_name)
        .replace("###ENUM_FN_HANDLER###", &enum_fn_handler);

    // structure
    if let EnumType::Structure { mut_token, .. } = data {
        let structure_handle = template_mv
            .find_struct_def_by_name(IdentStr::new(name)?)
            .ok_or_else(|| anyhow!("{name:?} not found in the module"))?;
        let enum_index_handle = structure_handle.struct_handle.to_string();
        let enum_index_definition = template_mv
            .struct_defs
            .iter()
            .position(|item| item == structure_handle)
            .ok_or_else(|| anyhow!("{name:?} not found in the module"))?
            .to_string();

        result += TEMPLATE_ENUM_STRUCTURE
            .replace("###ENUM_NAME###", name)
            .replace(
                "###ENUM_INDEX_TOKEN###",
                if mut_token {
                    TEMPLATE_ENUM_TOKEN_MUT
                } else {
                    TEMPLATE_ENUM_TOKEN
                },
            )
            .replace("###ENUM_INDEX_HANDLE###", &enum_index_handle)
            .replace("###ENUM_INDEX_DEFINITION###", &enum_index_definition)
            .as_str();
    }

    Ok(result)
}

fn find_function_index(template_mv: &CompiledModule, fn_name: &str) -> Option<usize> {
    template_mv
        .function_handles()
        .iter()
        .enumerate()
        .find(|(.., item)| template_mv.identifier_at(item.name).as_str() == fn_name)
        .map(|(index, ..)| index)
}

pub fn find_address_const(
    module: &CompiledModule,
    addr: AccountAddress,
) -> Option<ConstantPoolIndex> {
    module
        .constant_pool
        .iter()
        .enumerate()
        .find(|(_, c)| match c.type_ {
            SignatureToken::Address => c.data.as_slice() == addr.as_slice(),
            _ => false,
        })
        .map(|(id, _)| ConstantPoolIndex(id as u16))
}
