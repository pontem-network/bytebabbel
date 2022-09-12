use anyhow::Error;
use eth::abi::entries::AbiEntries;
use eth::bytecode::types::Function;
use move_binary_format::CompiledModule;
use std::fmt::Write;

pub fn move_interface(module: &CompiledModule, abi: &AbiEntries) -> Result<String, Error> {
    let mut buff = String::new();
    let id = module.self_id();

    writeln!(
        buff,
        "module {}::{} {{",
        id.address().short_str_lossless(),
        id.name()
    )?;

    write_constants(&mut buff)?;

    abi.entries
        .iter()
        .filter_map(|e| match e {
            eth::abi::entries::AbiEntry::Function(f) => Some(f),
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
        "public fun constructor(_account_address: &signer) {{}}"
    )?;
    Ok(())
}

fn write_function(buff: &mut String, fun: &Function, module: &CompiledModule) -> Result<(), Error> {
    writeln!(
        buff,
        "public fun {}(_account_address: &signer, _args: std::vector::Vrctor<u8>) {{}}"
    )?;
    Ok(())
}
