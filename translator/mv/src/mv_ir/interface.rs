use anyhow::Error;
use eth::abi::entries::{AbiEntries, Entry, FunctionData};
use eth::abi::inc_ret_param::Param;
use eth::bytecode::types::EthType;
use eth::Flags;
use move_binary_format::CompiledModule;
use std::fmt::Write;

pub fn move_interface(
    module: &CompiledModule,
    abi: &AbiEntries,
    flags: Flags,
) -> Result<String, Error> {
    let mut buff = String::new();
    let id = module.self_id();

    let addr = if flags.package_interface {
        "self".to_string()
    } else {
        format!("0x{}", id.address().short_str_lossless())
    };

    writeln!(buff, "module {}::{} {{", addr, id.name())?;

    write_constants(&mut buff)?;
    writeln!(buff)?;

    abi.entries
        .iter()
        .filter_map(|e| match e {
            Entry::Function(f) => Some(f),
            _ => None,
        })
        .map(|f| write_function(&mut buff, f, module, flags))
        .collect::<Result<Vec<_>, Error>>()?;

    if (flags.native_input || flags.native_output) && !flags.u128_io {
        write_u256(&mut buff)?;
    }
    writeln!(buff, "}}")?;
    Ok(buff)
}

fn write_constants(buff: &mut String) -> Result<(), Error> {
    writeln!(
        buff,
        "{:width$}public native fun constructor(account_address: &signer);",
        "",
        width = 4
    )?;
    Ok(())
}

fn write_function(
    buff: &mut String,
    fun: &FunctionData,
    _module: &CompiledModule,
    flags: Flags,
) -> Result<(), Error> {
    let args = if flags.native_input {
        if let Some(input) = &fun.inputs {
            input
                .iter()
                .map(|p| map_param(p, &flags))
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            String::default()
        }
    } else {
        "args: &vector<u8>".to_string()
    };

    let ret = if flags.native_output {
        if flags.hidden_output {
            "".to_string()
        } else if let Some(output) = &fun.outputs {
            let params = output
                .iter()
                .map(|p| map_type(&EthType::try_from(p).unwrap(), &flags))
                .collect::<Vec<_>>()
                .join(", ");
            if output.is_empty() {
                params
            } else if output.len() == 1 {
                format!(": {}", params)
            } else {
                format!(": ({})", params)
            }
        } else {
            String::default()
        }
    } else {
        ": vector<u8>".to_string()
    };

    writeln!(
        buff,
        "{:width$}public native fun {}(account_address: &signer,{}){};",
        "",
        fun.name.as_deref().unwrap_or("anonymous"),
        args,
        ret,
        width = 4
    )?;
    writeln!(buff)?;
    Ok(())
}

fn write_u256(buff: &mut String) -> Result<(), Error> {
    writeln!(
        buff,
        "{:width$}struct U256 has copy, drop, store {{",
        "",
        width = 4
    )?;
    writeln!(buff, "{:width$}v0: u64,", "", width = 8)?;
    writeln!(buff, "{:width$}v1: u64,", "", width = 8)?;
    writeln!(buff, "{:width$}v2: u64,", "", width = 8)?;
    writeln!(buff, "{:width$}v3: u64,", "", width = 8)?;
    writeln!(buff, "{:width$}}}", "", width = 4)?;
    writeln!(buff)?;

    writeln!(
        buff,
        "{:width$}public native fun as_u128(val: U256): u128;",
        "",
        width = 4
    )?;
    writeln!(buff)?;
    writeln!(
        buff,
        "{:width$}public native fun from_u128(val: u128): U256;",
        "",
        width = 4
    )?;
    Ok(())
}

fn map_param(p: &Param, flags: &Flags) -> String {
    let ty = map_type(&EthType::try_from(p).unwrap(), flags);
    format!("{}: {}", p.name, ty)
}

fn map_type(ty: &EthType, flags: &Flags) -> String {
    match ty {
        EthType::U256 => {
            if flags.u128_io {
                "u128"
            } else {
                "U256"
            }
        }
        EthType::Bool => "bool",
        EthType::Address => "address",
        EthType::Bytes => "vector<u8>",
    }
    .to_string()
}
