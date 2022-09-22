use std::fmt::Write;

use anyhow::Error;
use ethabi::{Contract, Function};
use move_binary_format::CompiledModule;

use eth::bytecode::types::EthType;
use eth::Flags;

pub fn move_interface(
    module: &CompiledModule,
    abi: &Contract,
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

    abi.functions()
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
    fun: &Function,
    _module: &CompiledModule,
    flags: Flags,
) -> Result<(), Error> {
    let args = if flags.native_input {
        fun.inputs
            .iter()
            .map(|p| map_param(p, &flags))
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        "args: &vector<u8>".to_string()
    };

    let ret = if flags.native_output {
        if flags.hidden_output {
            "".to_string()
        } else {
            let params = fun
                .outputs
                .iter()
                .map(|p| map_type(&EthType::try_from(p).unwrap(), &flags))
                .collect::<Vec<_>>()
                .join(", ");
            if fun.outputs.is_empty() {
                params
            } else if fun.outputs.len() == 1 {
                format!(": {}", params)
            } else {
                format!(": ({})", params)
            }
        }
    } else {
        ": vector<u8>".to_string()
    };

    writeln!(
        buff,
        "{:width$}public native fun {}(account_address: &signer,{}){};",
        "",
        fun.name,
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

fn map_param(p: &ethabi::Param, flags: &Flags) -> String {
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
