use std::fs;

use anyhow::{anyhow, Result};
use move_binary_format::access::ModuleAccess;
use move_binary_format::CompiledModule;
use move_core_types::identifier::IdentStr;

use crate::Paths;

const TEMPLATE_USE: &str = "\
use crate::Function;
use enum_iterator::Sequence;
use move_binary_format::file_format::{
    FunctionHandleIndex, SignatureToken, StructDefinitionIndex, StructHandleIndex,
};
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

impl ###ENUM_NAME### {
    pub fn token() -> SignatureToken {
        ###ENUM_INDEX_TOKEN###
    }

    pub fn instance() -> StructDefinitionIndex {
        StructDefinitionIndex(###ENUM_INDEX_DEFINITION###)
    }
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

const U256_TABLE: [(&str, &str); 35] = [
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
    ("SignExtend", "sexp"),
    ("Sar", "sar"),
    ("FromAddress", "from_address"),
    ("ToAddress", "to_address"),
    ("FromU128", "from_u128"),
    ("ToU128", "as_u128"),
];

pub(crate) fn generate_table(paths: &Paths) -> Result<()> {
    let template_mv_cont = fs::read(&paths.template_mv)?;
    let template = CompiledModule::deserialize(&template_mv_cont).unwrap();

    let result = [
        (false, "Memory", MEMORY_TABLE.to_vec()),
        (true, "Persist", PERSIST_TABLE.to_vec()),
        (false, "U256", U256_TABLE.to_vec()),
    ]
    .into_iter()
    .map(|(mut_token, struct_name, table)| gen_enum_code(&template, mut_token, struct_name, &table))
    .collect::<Result<Vec<String>>>()?
    .join("\n");

    fs::write(&paths.instrinsic_table, format!("{TEMPLATE_USE}{result}"))?;

    Ok(())
}

fn gen_enum_code(
    template_mv: &CompiledModule,
    mut_token: bool,
    struct_name: &str,
    table: &[(&str, &str)],
) -> Result<String> {
    let enum_body = table
        .iter()
        .map(|(name, ..)| format!("{name},"))
        .collect::<Vec<String>>()
        .join("\n    ");

    let structure_handle = template_mv
        .find_struct_def_by_name(IdentStr::new(struct_name)?)
        .ok_or_else(|| anyhow!("{struct_name:?} not found in the module"))?;
    let enum_index_handle = structure_handle.struct_handle.to_string();
    let enum_index_definition = template_mv
        .struct_defs
        .iter()
        .position(|item| item == structure_handle)
        .ok_or_else(|| anyhow!("{struct_name:?} not found in the module"))?
        .to_string();

    let enum_fn_name = table
        .iter()
        .map(|(name, move_fn)| format!("Self::{name} => \"{move_fn}\","))
        .collect::<Vec<String>>()
        .join("\n            ");

    let enum_fn_handler = table
        .iter()
        .map(|(name, move_fn)| {
            let fn_index = find_fundction_index(template_mv, move_fn)
                .ok_or_else(|| anyhow!("Function {move_fn:?} not found"))?;
            Ok(format!("Self::{name} => FunctionHandleIndex({fn_index}),"))
        })
        .collect::<Result<Vec<String>>>()?
        .join("\n            ");

    let result = TEMPLATE_ENUM
        .replace("###ENUM_NAME###", struct_name)
        .replace("###ENUM_BODY###", &enum_body)
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
        .replace("###ENUM_FN_NAME###", &enum_fn_name)
        .replace("###ENUM_FN_HANDLER###", &enum_fn_handler);

    Ok(result)
}

fn find_fundction_index(template_mv: &CompiledModule, fn_name: &str) -> Option<usize> {
    template_mv
        .function_handles()
        .iter()
        .enumerate()
        .find(|(.., item)| template_mv.identifier_at(item.name).as_str() == fn_name)
        .map(|(index, ..)| index)
}