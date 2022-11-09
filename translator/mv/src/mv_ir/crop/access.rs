use anyhow::Error;
use move_binary_format::{
    access::ModuleAccess,
    file_format::{Bytecode, Signature, SignatureToken, Visibility},
    file_format::{
        FieldHandleIndex, FieldInstantiationIndex, FunctionHandleIndex,
        StructDefInstantiationIndex, StructDefinitionIndex, StructHandleIndex, TableIndex,
    },
    CompiledModule,
};

use std::collections::HashSet;

pub fn find_all_functions(module: &CompiledModule) -> Result<HashSet<FunctionHandleIndex>, Error> {
    let mut used_functions: HashSet<FunctionHandleIndex> = HashSet::new();
    let mut queue: Vec<FunctionHandleIndex> = vec![];

    // get public unique calls
    for func in &module.function_defs {
        if func.visibility == Visibility::Public {
            used_functions.insert(func.function);
            queue.push(func.function);
        }
    }

    // insert "to_bytes"
    // there's a few handles "to_bytes"
    // we need handle with any ModuleHandle
    // TODO: transfer it to external template
    // find Module by name!
    let f = module
        .function_handles
        .iter()
        .enumerate()
        .filter(|(_, f)| module.identifier_at(f.name).as_str() == "to_bytes")
        .map(|(index, _)| index)
        .collect::<Vec<_>>();

    for pos in f {
        let handler = FunctionHandleIndex(pos as TableIndex);
        used_functions.insert(handler);
        queue.push(handler);
    }

    while let Some(f) = queue.pop() {
        let res = find_functions(module, f, &mut queue)?;
        for el in res {
            used_functions.insert(*el);
        }
    }

    Ok(used_functions)
}

fn find_functions<'a>(
    module: &CompiledModule,
    func_id: FunctionHandleIndex,
    set: &'a mut Vec<FunctionHandleIndex>,
) -> Result<&'a Vec<FunctionHandleIndex>, Error> {
    let mut iter_all_functions = module.function_defs.iter();
    let main_f_def = iter_all_functions.find(|&function_index| function_index.function == func_id);
    let main_f_def = match main_f_def {
        Some(fun_def) => fun_def,
        None => return Ok(set),
    };

    if let Some(code_unit) = &main_f_def.code {
        for code in &code_unit.code {
            let idx = match code {
                Bytecode::Call(idx) => *idx,
                Bytecode::CallGeneric(idx) => module.function_instantiation_at(*idx).handle,
                _ => continue,
            };
            if !set.contains(&idx) {
                set.push(idx);
            }
        }
    }

    Ok(set)
}

pub fn find_bytecode_fun_defs(
    module: &CompiledModule,
    byte_codes: &HashSet<Bytecode>,
) -> HashSet<Bytecode> {
    let mut set: HashSet<Bytecode> = HashSet::new();
    for def in module.function_defs.iter() {
        if let Some(code_unit) = &def.code {
            for code in code_unit.code.iter() {
                if byte_codes.contains(code) {
                    set.insert(code.clone());
                }
            }
        }
    }
    set
}

pub fn find_all_structs(module: &CompiledModule) -> Result<HashSet<StructHandleIndex>, Error> {
    let mut set: HashSet<StructHandleIndex> = HashSet::new();

    for def in module.function_defs.iter() {
        if let Some(code_unit) = &def.code {
            for code in code_unit.code.iter() {
                let def_idx = match code {
                    // defs
                    Bytecode::Pack(idx) => *idx,
                    Bytecode::Unpack(idx) => *idx,
                    Bytecode::MutBorrowGlobal(idx) => *idx,
                    Bytecode::ImmBorrowGlobal(idx) => *idx,
                    Bytecode::Exists(idx) => *idx,
                    Bytecode::MoveFrom(idx) => *idx,
                    Bytecode::MoveTo(idx) => *idx,

                    // instantiation
                    Bytecode::PackGeneric(insta) => module.struct_instantiation_at(*insta).def,
                    Bytecode::UnpackGeneric(insta) => module.struct_instantiation_at(*insta).def,
                    Bytecode::MutBorrowGlobalGeneric(insta) => {
                        module.struct_instantiation_at(*insta).def
                    }
                    Bytecode::ImmBorrowGlobalGeneric(insta) => {
                        module.struct_instantiation_at(*insta).def
                    }
                    Bytecode::ExistsGeneric(insta) => module.struct_instantiation_at(*insta).def,
                    Bytecode::MoveFromGeneric(insta) => module.struct_instantiation_at(*insta).def,
                    Bytecode::MoveToGeneric(insta) => module.struct_instantiation_at(*insta).def,

                    _ => continue,
                };

                set.insert(module.struct_def_at(def_idx).struct_handle);
            }
        }

        for el in def.acquires_global_resources.iter() {
            set.insert(module.struct_def_at(*el).struct_handle);
        }
    }

    // find signature
    let mut signatures: HashSet<&Signature> = HashSet::new();
    for fun_insta in module.function_instantiations.iter() {
        signatures.insert(module.signature_at(fun_insta.type_parameters));
    }

    for fun_handler in module.function_handles.iter() {
        signatures.insert(module.signature_at(fun_handler.parameters));
        signatures.insert(module.signature_at(fun_handler.return_));
    }

    for sign in signatures.iter() {
        for token in sign.0.iter() {
            let handle = match token {
                // handles
                SignatureToken::Struct(handle) => *handle,
                SignatureToken::StructInstantiation(handle, _v) => *handle,
                _ => continue,
            };

            set.insert(handle);
        }
    }

    Ok(set)
}

pub enum IndexType<'a> {
    StructDef(&'a mut StructDefinitionIndex),
    StructInsta(&'a mut StructDefInstantiationIndex),
    FieldHandle(&'a mut FieldHandleIndex),
    FieldInsta(&'a mut FieldInstantiationIndex),
    None,
}

pub fn match_bytecode(code: &mut Bytecode) -> IndexType {
    match code {
        Bytecode::Pack(idx) => IndexType::StructDef(idx),
        Bytecode::Unpack(idx) => IndexType::StructDef(idx),
        Bytecode::MutBorrowGlobal(idx) => IndexType::StructDef(idx),
        Bytecode::ImmBorrowGlobal(idx) => IndexType::StructDef(idx),
        Bytecode::Exists(idx) => IndexType::StructDef(idx),
        Bytecode::MoveFrom(idx) => IndexType::StructDef(idx),
        Bytecode::MoveTo(idx) => IndexType::StructDef(idx),

        Bytecode::PackGeneric(idx) => IndexType::StructInsta(idx),
        Bytecode::UnpackGeneric(idx) => IndexType::StructInsta(idx),
        Bytecode::MutBorrowGlobalGeneric(idx) => IndexType::StructInsta(idx),
        Bytecode::ImmBorrowGlobalGeneric(idx) => IndexType::StructInsta(idx),
        Bytecode::ExistsGeneric(idx) => IndexType::StructInsta(idx),
        Bytecode::MoveFromGeneric(idx) => IndexType::StructInsta(idx),
        Bytecode::MoveToGeneric(idx) => IndexType::StructInsta(idx),

        Bytecode::MutBorrowField(idx) => IndexType::FieldHandle(idx),
        Bytecode::ImmBorrowField(idx) => IndexType::FieldHandle(idx),

        Bytecode::MutBorrowFieldGeneric(idx) => IndexType::FieldInsta(idx),
        Bytecode::ImmBorrowFieldGeneric(idx) => IndexType::FieldInsta(idx),

        _ => IndexType::None,
    }
}
