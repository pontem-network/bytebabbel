use anyhow::Error;
use eth::abi::entries::{AbiEntries, Entry, FunctionData};
use move_binary_format::CompiledModule;
use std::fmt::Write;

pub fn move_interface(module: &CompiledModule, abi: &AbiEntries) -> Result<String, Error> {
    let mut buff = String::new();
    let id = module.self_id();

    writeln!(
        buff,
        "module 0x{}::{} {{",
        id.address().short_str_lossless(),
        id.name()
    )?;

    write_constants(&mut buff)?;

    abi.entries
        .iter()
        .filter_map(|e| match e {
            Entry::Function(f) => Some(f),
            _ => None,
        })
        .map(|f| write_function(&mut buff, f, module))
        .collect::<Result<Vec<_>, Error>>()?;
    writeln!(buff, "}}")?;
    Ok(buff)
}

fn write_constants(buff: &mut String) -> Result<(), Error> {
    writeln!(
        buff,
        "{:width$}public fun constructor(_account_address: &signer) {{}}",
        "",
        width = 4
    )?;
    Ok(())
}

fn write_function(
    buff: &mut String,
    fun: &FunctionData,
    _module: &CompiledModule,
) -> Result<(), Error> {
    writeln!(
        buff,
        "{:width$}public fun {}(_account_address: &signer, _args: vector<u8>): vector<u8> {{",
        "",
        fun.name.as_deref().unwrap_or("anonymous"),
        width = 4
    )?;
    writeln!(
        buff,
        "{:width$}return std::vector::empty<u8>()",
        "",
        width = 8
    )?;
    writeln!(buff, "{:width$}}}", "", width = 4)?;
    Ok(())
}
