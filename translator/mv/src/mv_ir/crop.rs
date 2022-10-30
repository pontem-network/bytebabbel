use anyhow::{anyhow, Error};
use move_binary_format::{
    access::ModuleAccess,
    file_format::{Bytecode, FunctionInstantiationIndex},
    file_format::{ConstantPoolIndex, FunctionHandleIndex, TableIndex},
    file_format::{SignatureToken, StructDefinitionIndex, StructFieldInformation, Visibility},
    CompiledModule,
};

use crate::translator::identifier::IdentidierWriter;
use crate::translator::signature::SignatureWriter;

use move_binary_format::internals::ModuleIndex;
use std::collections::{HashMap, HashSet, VecDeque};

use intrinsic::{table::Info, Function};

/// remove unused resources in module
pub fn crop(module: &mut CompiledModule) -> Result<(), Error> {
    // function_defs, function_handles, function_instantiations
    let set = find_all_functions(module)?;

    // generate vec of FunctionHandleIndexes to delete
    let indexes_to_delete: Vec<FunctionHandleIndex> = (0..module.function_handles.len())
        .map(|x| FunctionHandleIndex(x as u16))
        .filter(|idx| !set.contains(idx))
        .collect();

    remove_functions(module, &indexes_to_delete)?;

    // constant_pool
    let all_constants: HashSet<Bytecode> = HashSet::from_iter(
        (0..module.constant_pool.len()).map(|x| Bytecode::LdConst(ConstantPoolIndex(x as u16))),
    );
    let s = find_bytecode_fun_defs(module, &all_constants);
    let constants_to_remove: HashSet<ConstantPoolIndex> = all_constants
        .difference(&s)
        .map(|x| {
            if let Bytecode::LdConst(index) = x {
                *index
            } else {
                // never get here
                ConstantPoolIndex(0)
            }
        })
        .collect();

    remove_constants(module, &Vec::from_iter(constants_to_remove))?;

    // structs
    // let set = find_all_structs(module)?;
    // let all_structs: HashSet<StructDefinitionIndex> = HashSet::from_iter(
    //     (0..module.struct_defs.len()).map(|x| StructDefinitionIndex(x as TableIndex)),
    // );
    // let mut struct_index_to_delete: Vec<StructDefinitionIndex> =
    //     Vec::from_iter(all_structs.difference(&set).copied());
    // struct_index_to_delete.sort();

    // remove_structs(module, &struct_index_to_delete)?;

    // identifiers
    reidex_indetifiers(module)?;

    // signature
    reidex_signatures(module)?;

    Ok(())
}

fn remove_functions(
    module: &mut CompiledModule,
    indexes_to_delete: &[FunctionHandleIndex],
) -> Result<(), Error> {
    // empty function
    let a = Info::ToDelete;
    let handler_to_delete_index = a.handler();
    let handler_to_delete = (*module.function_handle_at(handler_to_delete_index)).clone();

    // mark handlers to delete
    for el in indexes_to_delete {
        // defs
        let pos = module.function_defs.iter().position(|x| x.function == *el);
        if let Some(position) = pos {
            module.function_defs[position].function = handler_to_delete_index;
        }
        // instantiations
        let pos = module
            .function_instantiations
            .iter()
            .position(|x| x.handle == *el);
        if let Some(position) = pos {
            // TODO
            let _sign_index = module.function_instantiations[position].type_parameters;

            module.function_instantiations[position].handle = handler_to_delete_index
        }
        // signatures
        // TODO
        // let identifier_index_to_delete = module.function_handles[el.into_index()].name;
        // let (_sign_param, _sign_return) = (
        //     module.function_handles[el.into_index()].parameters,
        //     module.function_handles[el.into_index()].return_,
        // );
        // // identifiers
        // module.identifiers[identifier_index_to_delete.0 as usize] = Identifier::from_str("<SELF>")?;
        // handles
        module.function_handles[el.into_index()] = handler_to_delete.clone();
    }

    module
        .function_defs
        .retain(|f| f.function != handler_to_delete_index);

    // delete all marked handles
    let mut index_transaction: HashMap<FunctionHandleIndex, FunctionHandleIndex> = HashMap::new();
    let mut last_not_delete: FunctionHandleIndex = FunctionHandleIndex(0);

    for (indx, func_handler) in module.function_handles.iter().enumerate() {
        if *func_handler != handler_to_delete {
            index_transaction.insert(FunctionHandleIndex(indx as TableIndex), last_not_delete);
            last_not_delete.0 += 1;
        }
    }
    module.function_handles.retain(|f| *f != handler_to_delete);

    // change Calls in function defs
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

    // instations
    let mut insta_index_transaction: HashMap<
        FunctionInstantiationIndex,
        FunctionInstantiationIndex,
    > = HashMap::new();
    let mut last_not_delete: FunctionInstantiationIndex = FunctionInstantiationIndex(0);

    for (indx, func_instation) in module.function_instantiations.iter().enumerate() {
        if func_instation.handle != handler_to_delete_index {
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

    // change CallGeneric in function defs
    for def in module.function_defs.iter_mut() {
        if let Some(ref mut code_unit) = def.code {
            for code in code_unit.code.iter_mut() {
                if let Bytecode::CallGeneric(func_insta_index) = code {
                    let new_index = match insta_index_transaction.get(func_insta_index) {
                        Some(idx) => *idx,
                        None => {
                            return Err(anyhow!(
                                "Error while changing CallGeneric(function_instantiation) opcode:\nno instation for {:?}",
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

    // is it ok to use empty vector to mark constant to delete?
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

#[allow(dead_code)]
fn remove_structs(
    module: &mut CompiledModule,
    indexes_to_delete: &[StructDefinitionIndex],
) -> Result<(), Error> {
    let mut index_transaction: HashMap<StructDefinitionIndex, StructDefinitionIndex> =
        HashMap::new();
    let mut last_not_delete: StructDefinitionIndex = StructDefinitionIndex(0);

    for indx in 0..module.struct_defs.len() {
        if !indexes_to_delete.contains(&StructDefinitionIndex(indx as TableIndex)) {
            index_transaction.insert(StructDefinitionIndex(indx as TableIndex), last_not_delete);
            last_not_delete.0 += 1;
        }
    }

    // TODO: also delete handles
    // for indx in indexes_to_delete.iter().rev() {
    //     let handle_index = module.struct_defs[indx.into_index()].struct_handle.0 as usize;
    //     module.struct_handles[handle_index] = StructHandle {};
    // }

    for indx in indexes_to_delete.iter().rev() {
        module.struct_defs.remove(indx.into_index());
    }

    for def in module.function_defs.iter_mut() {
        if let Some(ref mut code_unit) = def.code {
            for code in code_unit.code.iter_mut() {
                let el = match code {
                    Bytecode::Pack(idx) => idx,
                    Bytecode::Unpack(idx) => idx,
                    Bytecode::MutBorrowGlobal(idx) => idx,
                    Bytecode::ImmBorrowGlobal(idx) => idx,
                    Bytecode::Exists(idx) => idx,
                    Bytecode::MoveFrom(idx) => idx,
                    Bytecode::MoveTo(idx) => idx,
                    _ => continue,
                };
                *el = match index_transaction.get(el) {
                    Some(idx) => *idx,
                    None => continue,
                };
            }
        }

        for el in def.acquires_global_resources.iter_mut() {
            *el = match index_transaction.get(el) {
                Some(idx) => *idx,
                None => continue,
            };
        }
    }

    for el in module.struct_def_instantiations.iter_mut() {
        el.def = match index_transaction.get(&el.def) {
            Some(idx) => *idx,
            None => continue,
        };
    }

    for field_handler in module.field_handles.iter_mut() {
        field_handler.owner = match index_transaction.get(&field_handler.owner) {
            Some(idx) => *idx,
            None => continue,
        };
    }

    Ok(())
}

fn reidex_indetifiers(module: &mut CompiledModule) -> Result<(), Error> {
    let mut writer = IdentidierWriter::new(&[]);
    // TODO: delete clone
    // problem: cannot move module.identifiers because of mut ref
    let old_identidiers = module.identifiers.clone();

    // module_handles
    for module_handle in module.module_handles.iter_mut() {
        let ident = &old_identidiers[module_handle.name.into_index()];
        module_handle.name = writer.make_identifier(ident.as_str())?;
    }

    // fun handles
    for fun_handler in module.function_handles.iter_mut() {
        let ident = &old_identidiers[fun_handler.name.into_index()];
        fun_handler.name = writer.make_identifier(ident.as_str())?;
    }

    // struct handles
    for struct_handler in module.struct_handles.iter_mut() {
        let ident = &old_identidiers[struct_handler.name.into_index()];
        struct_handler.name = writer.make_identifier(ident.as_str())?;
    }

    // field defs
    for struct_def in module.struct_defs.iter_mut() {
        let field_defs = match &mut struct_def.field_information {
            StructFieldInformation::Native => continue,
            StructFieldInformation::Declared(field_definitions) => field_definitions,
        };

        for field_def in field_defs.iter_mut() {
            let ident = &old_identidiers[field_def.name.into_index()];
            field_def.name = writer.make_identifier(ident.as_str())?;
        }
    }

    module.identifiers = writer.freeze();

    Ok(())
}

fn reidex_signatures(module: &mut CompiledModule) -> Result<(), Error> {
    let mut writer = SignatureWriter::new(&[]);
    // TODO: delete clone
    // problem: cannot move module.identifiers because of mut ref
    let old_signatures = module.signatures.clone();

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

// Find functions

// TODO change VecDeque to another structure
fn find_all_functions(module: &CompiledModule) -> Result<HashSet<FunctionHandleIndex>, Error> {
    let mut used_functions: HashSet<FunctionHandleIndex> = HashSet::new();
    let mut queue: VecDeque<FunctionHandleIndex> = VecDeque::new();

    // get public unique calls
    for func in &module.function_defs {
        if func.visibility == Visibility::Public {
            used_functions.insert(func.function);
            queue.push_back(func.function);
        }
    }

    // insert "to_bytes"
    // there's a few handles "to_bytes"
    // we need handle with any ModuleHandle
    // TODO: delete constant this constant
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
        queue.push_back(handler);
    }

    loop {
        if let Some(f) = queue.pop_front() {
            let res = find_functions(module, f, &mut queue)?;

            for el in res {
                used_functions.insert(*el);
            }

            if queue.is_empty() {
                break;
            }
        }
    }

    Ok(used_functions)
}

fn find_functions<'a>(
    module: &CompiledModule,
    func_id: FunctionHandleIndex,
    set: &'a mut VecDeque<FunctionHandleIndex>,
) -> Result<&'a VecDeque<FunctionHandleIndex>, Error> {
    // TODO: change Vec to HashSet
    let mut iter_all_functions = module.function_defs.iter();
    // let mut set: Vec<FunctionHandleIndex> = vec![];

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
                set.push_back(idx);
            }
        }
    }

    Ok(set)
}

// we need to generalize this function for find, swap any item
fn find_bytecode_fun_defs(
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

#[allow(dead_code)]
fn find_all_structs(module: &CompiledModule) -> Result<HashSet<StructDefinitionIndex>, Error> {
    let mut set: HashSet<StructDefinitionIndex> = HashSet::new();

    for def in module.function_defs.iter() {
        if let Some(code_unit) = &def.code {
            for code in code_unit.code.iter() {
                let el = match code {
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

                set.insert(el);
            }
        }

        for el in def.acquires_global_resources.iter() {
            set.insert(*el);
        }
    }

    // TODO: search in cropped signatures pool
    for sign in module.signatures.iter() {
        for token in &sign.0 {
            let handle = match token {
                // handles
                SignatureToken::Struct(handle) => *handle,
                SignatureToken::StructInstantiation(handle, _v) => *handle,
                _ => continue,
            };

            let pos = module
                .struct_defs
                .iter()
                .position(|def| def.struct_handle == handle);
            match pos {
                Some(position) => {
                    set.insert(StructDefinitionIndex(position as TableIndex));
                }
                None => continue,
            }
        }
    }

    Ok(set)
}
