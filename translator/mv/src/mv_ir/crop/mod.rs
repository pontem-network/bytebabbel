use crate::translator::{identifier::IdentifierWriter, signature::SignatureWriter};
use anyhow::{anyhow, Error};
use move_binary_format::file_format::SignatureIndex;
use move_binary_format::internals::ModuleIndex;
use move_binary_format::{
    access::ModuleAccess,
    file_format::{
        AbilitySet, Bytecode, FunctionHandle, SignatureToken, StructFieldInformation, StructHandle,
    },
    file_format::{
        ConstantPoolIndex, FieldHandleIndex, FieldInstantiationIndex, FunctionHandleIndex,
        FunctionInstantiationIndex, IdentifierIndex, ModuleHandleIndex,
        StructDefInstantiationIndex, StructDefinitionIndex, StructHandleIndex, TableIndex,
    },
    CompiledModule,
};
use move_core_types::identifier::Identifier;

use std::collections::{HashMap, HashSet};
use std::mem;

mod access;

use access::IndexType;

/// remove unused resources in module
pub fn crop(module: &mut CompiledModule) -> Result<(), Error> {
    let set = access::find_all_functions(module)?;
    let indexes_to_delete: Vec<FunctionHandleIndex> = (0..module.function_handles.len())
        .map(|x| FunctionHandleIndex(x as u16))
        .filter(|idx| !set.contains(idx))
        .collect();

    remove_functions(module, &indexes_to_delete)?;

    let all_constants: HashSet<Bytecode> = HashSet::from_iter(
        (0..module.constant_pool.len()).map(|x| Bytecode::LdConst(ConstantPoolIndex(x as u16))),
    );
    let s = access::find_bytecode_fun_defs(module, &all_constants);
    let constants_to_remove: HashSet<ConstantPoolIndex> = all_constants
        .difference(&s)
        .map(|x| {
            if let Bytecode::LdConst(index) = x {
                *index
            } else {
                unreachable!();
            }
        })
        .collect();

    remove_constants(module, &Vec::from_iter(constants_to_remove))?;

    let set = access::find_all_structs(module)?;
    let all_structs: HashSet<StructHandleIndex> = HashSet::from_iter(
        (0..module.struct_handles.len()).map(|x| StructHandleIndex(x as TableIndex)),
    );
    let mut struct_index_to_delete: Vec<StructHandleIndex> =
        Vec::from_iter(all_structs.difference(&set).copied());
    struct_index_to_delete.sort();

    remove_structs(module, &struct_index_to_delete)?;

    reindex_identifiers(module)?;

    reindex_signatures(module)?;

    Ok(())
}

fn remove_functions(
    module: &mut CompiledModule,
    indexes_to_delete: &[FunctionHandleIndex],
) -> Result<(), Error> {
    // empty function
    let handler_to_delete = FunctionHandle {
        module: ModuleHandleIndex(0),
        name: IdentifierIndex(module.identifiers.len() as TableIndex),
        parameters: SignatureIndex(0),
        return_: SignatureIndex(0),
        type_parameters: vec![],
    };
    let handler_to_delete_index = FunctionHandleIndex(module.function_handles.len() as TableIndex);
    module.function_handles.push(handler_to_delete.clone());
    module
        .identifiers
        .push(Identifier::new("function_to_delete".to_string())?);

    // mark handlers to delete
    for el in indexes_to_delete {
        let pos = module.function_defs.iter().position(|x| x.function == *el);
        if let Some(position) = pos {
            module.function_defs[position].function = handler_to_delete_index;
        }

        let pos = module
            .function_instantiations
            .iter()
            .position(|x| x.handle == *el);
        if let Some(position) = pos {
            module.function_instantiations[position].handle = handler_to_delete_index
        }

        module.function_handles[el.into_index()] = handler_to_delete.clone();
    }

    module
        .function_defs
        .retain(|f| f.function != handler_to_delete_index);

    let mut index_transaction: HashMap<FunctionHandleIndex, FunctionHandleIndex> = HashMap::new();
    let mut last_not_delete: FunctionHandleIndex = FunctionHandleIndex(0);
    for (indx, func_handler) in module.function_handles.iter().enumerate() {
        if *func_handler != handler_to_delete {
            index_transaction.insert(FunctionHandleIndex(indx as TableIndex), last_not_delete);
            last_not_delete.0 += 1;
        }
    }
    module.function_handles.retain(|f| *f != handler_to_delete);

    for def in module.function_defs.iter_mut() {
        def.function = match index_transaction.get(&def.function) {
            Some(idx) => *idx,
            None => {
                return Err(anyhow!(
                    "Error while removing function_handles:\nno handler for {:?}",
                    def.function
                ))
            }
        };

        if let Some(ref mut code_unit) = def.code {
            for code in code_unit.code.iter_mut() {
                if let Bytecode::Call(func_handle) = code {
                    let new_handle = match index_transaction.get(func_handle) {
                        Some(idx) => *idx,
                        None => {
                            return Err(anyhow!(
                                "Error while changing Call(function_handle) opcode:\nno handler for {:?}",
                                def.function
                            ))
                        }
                    };
                    *code = Bytecode::Call(new_handle);
                }
            }
        }
    }

    let mut insta_index_transaction: HashMap<
        FunctionInstantiationIndex,
        FunctionInstantiationIndex,
    > = HashMap::new();
    let mut last_not_delete: FunctionInstantiationIndex = FunctionInstantiationIndex(0);
    for (indx, func_insta) in module.function_instantiations.iter().enumerate() {
        if func_insta.handle != handler_to_delete_index {
            insta_index_transaction.insert(
                FunctionInstantiationIndex(indx as TableIndex),
                last_not_delete,
            );
            last_not_delete.0 += 1;
        }
    }

    module
        .function_instantiations
        .retain(|f| f.handle != handler_to_delete_index);

    for inst in module.function_instantiations.iter_mut() {
        inst.handle = match index_transaction.get(&inst.handle) {
            Some(handle) => *handle,
            None => {
                return Err(anyhow!(
                    "Error while removing function_handles:\nno handler for {:?}",
                    inst.handle
                ))
            }
        };
    }

    for def in module.function_defs.iter_mut() {
        if let Some(ref mut code_unit) = def.code {
            for code in code_unit.code.iter_mut() {
                if let Bytecode::CallGeneric(func_insta_index) = code {
                    let new_index = match insta_index_transaction.get(func_insta_index) {
                        Some(idx) => *idx,
                        None => {
                            return Err(anyhow!(
                                "Error while changing CallGeneric(function_instantiation) opcode:\nno instantiation for {:?}",
                                func_insta_index
                            ))
                        }
                    };
                    *code = Bytecode::CallGeneric(new_index);
                }
            }
        }
    }

    Ok(())
}

fn remove_constants(
    module: &mut CompiledModule,
    constants_to_delete: &[ConstantPoolIndex],
) -> Result<(), Error> {
    let mut index_transaction: HashMap<ConstantPoolIndex, ConstantPoolIndex> = HashMap::new();
    let mut last_not_delete: ConstantPoolIndex = ConstantPoolIndex(0);

    // mark constants to delete
    for indx in constants_to_delete {
        module.constant_pool[indx.0 as usize].data = vec![];
    }

    for (indx, constant) in module.constant_pool.iter().enumerate() {
        if !constant.data.is_empty() {
            index_transaction.insert(ConstantPoolIndex(indx as TableIndex), last_not_delete);
            last_not_delete.0 += 1;
        }
    }

    module.constant_pool.retain(|c| !c.data.is_empty());

    for def in module.function_defs.iter_mut() {
        if let Some(ref mut code_unit) = def.code {
            for code in code_unit.code.iter_mut() {
                if let Bytecode::LdConst(const_index) = code {
                    let new_index = match index_transaction.get(const_index) {
                        Some(idx) => *idx,
                        None => {
                            return Err(anyhow!(
                                "Error while changing LdConst(constant_index) opcode:\nno index for {:?}",
                                const_index
                            ))
                        }
                    };
                    *code = Bytecode::LdConst(new_index);
                }
            }
        }
    }

    Ok(())
}

fn remove_structs(
    module: &mut CompiledModule,
    indexes_to_delete: &[StructHandleIndex],
) -> Result<(), Error> {
    let handler_to_delete = StructHandle {
        module: ModuleHandleIndex(0),
        name: IdentifierIndex(module.identifiers.len() as TableIndex),
        abilities: AbilitySet::EMPTY,
        type_parameters: vec![],
    };
    let handler_to_delete_index = StructHandleIndex(module.struct_handles.len() as TableIndex);
    module.struct_handles.push(handler_to_delete.clone());
    module
        .identifiers
        .push(Identifier::new("struct_to_delete".to_string())?);

    for indx in indexes_to_delete.iter() {
        let pos = module
            .struct_defs
            .iter()
            .position(|x| x.struct_handle == *indx);
        if let Some(def_position) = pos {
            module.struct_defs[def_position].struct_handle = handler_to_delete_index;
        }
        module.struct_handles[indx.into_index()] = handler_to_delete.clone();
    }

    let mut defs_index_transaction: HashMap<StructDefinitionIndex, StructDefinitionIndex> =
        HashMap::new();
    let mut last_not_delete: StructDefinitionIndex = StructDefinitionIndex(0);
    for (indx, el) in module.struct_defs.iter().enumerate() {
        if el.struct_handle != handler_to_delete_index {
            defs_index_transaction
                .insert(StructDefinitionIndex(indx as TableIndex), last_not_delete);
            last_not_delete.0 += 1;
        }
    }

    let mut insta_index_transaction: HashMap<
        StructDefInstantiationIndex,
        StructDefInstantiationIndex,
    > = HashMap::new();
    let mut last_not_delete: StructDefInstantiationIndex = StructDefInstantiationIndex(0);
    for (indx, el) in module.struct_def_instantiations.iter().enumerate() {
        if module.struct_def_at(el.def).struct_handle != handler_to_delete_index {
            insta_index_transaction.insert(
                StructDefInstantiationIndex(indx as TableIndex),
                last_not_delete,
            );
            last_not_delete.0 += 1;
        }
    }

    let mut handle_index_transaction: HashMap<StructHandleIndex, StructHandleIndex> =
        HashMap::new();
    let mut last_not_delete: StructHandleIndex = StructHandleIndex(0);
    for indx in 0..module.struct_handles.len() {
        if !indexes_to_delete.contains(&StructHandleIndex(indx as TableIndex)) {
            handle_index_transaction.insert(StructHandleIndex(indx as TableIndex), last_not_delete);
            last_not_delete.0 += 1;
        }
    }

    let mut field_index_transaction: HashMap<FieldHandleIndex, FieldHandleIndex> = HashMap::new();
    let mut last_not_delete: FieldHandleIndex = FieldHandleIndex(0);
    for (indx, el) in module.field_handles.iter().enumerate() {
        if module.struct_def_at(el.owner).struct_handle != handler_to_delete_index {
            field_index_transaction.insert(FieldHandleIndex(indx as TableIndex), last_not_delete);
            last_not_delete.0 += 1;
        }
    }

    let mut field_insta_index_transaction: HashMap<
        FieldInstantiationIndex,
        FieldInstantiationIndex,
    > = HashMap::new();
    let mut last_not_delete: FieldInstantiationIndex = FieldInstantiationIndex(0);
    for (indx, el) in module.field_instantiations.iter().enumerate() {
        if module
            .struct_def_at(module.field_handle_at(el.handle).owner)
            .struct_handle
            != handler_to_delete_index
        {
            field_insta_index_transaction
                .insert(FieldInstantiationIndex(indx as TableIndex), last_not_delete);
            last_not_delete.0 += 1;
        }
    }

    module.struct_def_instantiations.retain(|insta| {
        module.struct_defs[insta.def.into_index()].struct_handle != handler_to_delete_index
    });

    module.field_instantiations.retain(|handle| {
        module.struct_defs[module.field_handles[handle.handle.into_index()]
            .owner
            .into_index()]
        .struct_handle
            != handler_to_delete_index
    });

    module.field_handles.retain(|handle| {
        module.struct_defs[handle.owner.into_index()].struct_handle != handler_to_delete_index
    });

    module
        .struct_defs
        .retain(|def| def.struct_handle != handler_to_delete_index);

    module
        .struct_handles
        .retain(|handle| *handle != handler_to_delete);

    // change all indexes
    for insta in module.struct_def_instantiations.iter_mut() {
        insta.def = match defs_index_transaction.get(&insta.def) {
            Some(idx) => *idx,
            None => {
                return Err(anyhow!(
                    "Error while changing structs:\nno def for {:?} (struct instantiations)",
                    insta
                ))
            }
        };
    }

    for field_insta in module.field_instantiations.iter_mut() {
        field_insta.handle = match field_index_transaction.get(&field_insta.handle) {
            Some(idx) => *idx,
            None => {
                return Err(anyhow!(
                    "Error while changing structs:\nno field_insta for {:?}",
                    field_insta
                ));
            }
        };
    }

    for field_handler in module.field_handles.iter_mut() {
        field_handler.owner = match defs_index_transaction.get(&field_handler.owner) {
            Some(idx) => *idx,
            None => {
                return Err(anyhow!(
                    "Error while changing structs:\nno field_handler for {:?}",
                    field_handler
                ));
            }
        };
    }

    for def in module.struct_defs.iter_mut() {
        def.struct_handle = match handle_index_transaction.get(&def.struct_handle) {
            Some(idx) => *idx,
            None => {
                return Err(anyhow!(
                    "Error while changing structs:\nno def for {:?}",
                    def
                ))
            }
        };
    }

    // change bytecode
    for def in module.function_defs.iter_mut() {
        if let Some(ref mut code_unit) = def.code {
            for code in code_unit.code.iter_mut() {
                match access::match_bytecode(code) {
                    IndexType::StructDef(indx) => {
                        *indx = match defs_index_transaction.get(indx) {
                            Some(idx) => *idx,
                            None => {
                                return Err(anyhow!(
                                    "Error while changing structs:\nno def for {:?}",
                                    indx
                                ))
                            }
                        };
                    }

                    IndexType::StructInsta(indx) => {
                        *indx = match insta_index_transaction.get(indx) {
                            Some(idx) => *idx,
                            None => {
                                return Err(anyhow!(
                                    "Error while changing structs:\nno insta for {:?}",
                                    indx
                                ))
                            }
                        };
                    }

                    IndexType::FieldHandle(indx) => {
                        *indx = match field_index_transaction.get(indx) {
                            Some(idx) => *idx,
                            None => {
                                return Err(anyhow!(
                                    "Error while changing structs:\nno field handler for {:?}",
                                    indx
                                ))
                            }
                        };
                    }

                    IndexType::FieldInsta(indx) => {
                        *indx = match field_insta_index_transaction.get(indx) {
                            Some(idx) => *idx,
                            None => {
                                return Err(anyhow!(
                                    "Error while changing structs:\nno field insta for {:?}",
                                    indx
                                ))
                            }
                        };
                    }

                    IndexType::None => continue,
                }
            }
        }

        for el in def.acquires_global_resources.iter_mut() {
            *el = match defs_index_transaction.get(el) {
                Some(idx) => *idx,
                None => {
                    return Err(anyhow!(
                        "Error while changing structs:\nno def for {:?} (global resources)",
                        el
                    ))
                }
            };
        }
    }

    // change signatures
    for signature in module.signatures.iter_mut() {
        for token in signature.0.iter_mut() {
            change_signature(token, &handle_index_transaction, None);
        }
    }

    for def in module.struct_defs.iter_mut() {
        match def.field_information {
            StructFieldInformation::Native => continue,
            StructFieldInformation::Declared(ref mut v) => {
                for field_def in v.iter_mut() {
                    change_signature(&mut field_def.signature.0, &handle_index_transaction, None);
                }
            }
        }
    }

    Ok(())
}

// now we iter through the whole signature_pool, we can get unused signature
fn change_signature(
    token: &mut SignatureToken,
    handle_index_transaction: &HashMap<StructHandleIndex, StructHandleIndex>,
    _last_handle_index: Option<StructHandleIndex>,
) {
    match token {
        SignatureToken::Struct(struct_handle_index) => {
            if let Some(&new_index) = handle_index_transaction.get(struct_handle_index) {
                *struct_handle_index = new_index;
            }
        }

        SignatureToken::StructInstantiation(struct_handle_index, signs) => {
            if let Some(&new_index) = handle_index_transaction.get(struct_handle_index) {
                let old_index = *struct_handle_index;
                *struct_handle_index = new_index;
                for sign in signs.iter_mut() {
                    change_signature(sign, handle_index_transaction, Some(old_index));
                }
            }
        }

        SignatureToken::Reference(sign) => {
            change_signature(sign, handle_index_transaction, _last_handle_index)
        }

        SignatureToken::MutableReference(sign) => {
            change_signature(sign, handle_index_transaction, _last_handle_index)
        }

        SignatureToken::Vector(sign) => {
            change_signature(sign, handle_index_transaction, _last_handle_index)
        }

        _ => {}
    }
}

fn reindex_identifiers(module: &mut CompiledModule) -> Result<(), Error> {
    let mut writer = IdentifierWriter::new(&[]);
    let old_identifiers = mem::take(&mut module.identifiers);

    for module_handle in module.module_handles.iter_mut() {
        let ident = &old_identifiers[module_handle.name.into_index()];
        module_handle.name = writer.make_identifier(ident.as_str())?;
    }

    for fun_handler in module.function_handles.iter_mut() {
        let ident = &old_identifiers[fun_handler.name.into_index()];
        fun_handler.name = writer.make_identifier(ident.as_str())?;
    }

    for struct_handler in module.struct_handles.iter_mut() {
        let ident = &old_identifiers[struct_handler.name.into_index()];
        struct_handler.name = writer.make_identifier(ident.as_str())?;
    }

    for struct_def in module.struct_defs.iter_mut() {
        let field_defs = match &mut struct_def.field_information {
            StructFieldInformation::Native => continue,
            StructFieldInformation::Declared(field_definitions) => field_definitions,
        };

        for field_def in field_defs.iter_mut() {
            let ident = &old_identifiers[field_def.name.into_index()];
            field_def.name = writer.make_identifier(ident.as_str())?;
        }
    }

    module.identifiers = writer.freeze();

    Ok(())
}

fn reindex_signatures(module: &mut CompiledModule) -> Result<(), Error> {
    let mut writer = SignatureWriter::new(&[]);
    let old_signatures = mem::take(&mut module.signatures);

    for fun_insta in module.function_instantiations.iter_mut() {
        let sign = &old_signatures[fun_insta.type_parameters.into_index()];
        let new_index = writer.make_signature(sign.0.to_vec());
        fun_insta.type_parameters = new_index;
    }

    for fun_handler in module.function_handles.iter_mut() {
        let sign = &old_signatures[fun_handler.parameters.into_index()];
        let new_index = writer.make_signature(sign.0.to_vec());
        fun_handler.parameters = new_index;

        let sign = &old_signatures[fun_handler.return_.into_index()];
        let new_index = writer.make_signature(sign.0.to_vec());
        fun_handler.return_ = new_index;
    }

    for struct_def_insta in module.struct_def_instantiations.iter_mut() {
        let sign = &old_signatures[struct_def_insta.type_parameters.into_index()];
        let new_index = writer.make_signature(sign.0.to_vec());
        struct_def_insta.type_parameters = new_index;
    }

    for field_insta in module.field_instantiations.iter_mut() {
        let sign = &old_signatures[field_insta.type_parameters.into_index()];
        let new_index = writer.make_signature(sign.0.to_vec());
        field_insta.type_parameters = new_index;
    }

    for fun_def in module.function_defs.iter_mut() {
        if let Some(code_unit) = &mut fun_def.code {
            let sign = &old_signatures[code_unit.locals.into_index()];
            let new_index = writer.make_signature(sign.0.to_vec());
            code_unit.locals = new_index;

            for code in &mut code_unit.code {
                let idx = match code {
                    Bytecode::VecPack(idx, _len) => idx,
                    Bytecode::VecLen(idx) => idx,
                    Bytecode::VecImmBorrow(idx) => idx,
                    Bytecode::VecMutBorrow(idx) => idx,
                    Bytecode::VecPushBack(idx) => idx,
                    Bytecode::VecPopBack(idx) => idx,
                    Bytecode::VecUnpack(idx, _len) => idx,
                    Bytecode::VecSwap(idx) => idx,
                    _ => continue,
                };
                let sign = &old_signatures[idx.into_index()];
                let new_index = writer.make_signature(sign.0.to_vec());
                *idx = new_index;
            }
        }
    }

    module.signatures = writer.freeze();

    Ok(())
}
