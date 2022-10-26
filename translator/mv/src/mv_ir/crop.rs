use anyhow::{anyhow, Error};
use move_binary_format::{
    access::ModuleAccess,
    file_format::Visibility,
    file_format::{Bytecode, FunctionInstantiationIndex},
    file_format::{ConstantPoolIndex, FunctionHandleIndex, ModuleHandleIndex, TableIndex},
    CompiledModule,
};

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
    // fields
    // modules

    // signature
    // identifiers

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
            module.function_instantiations[position].handle = handler_to_delete_index
        }
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

    // insert "to_bytes"
    // there's a few handles "to_bytes"
    // we need handle with ModuleHandleIndex(4) - aptos_coin
    // TODO: delete constant this constant
    // find Module by name!
    let f = module.function_handles.iter().position(|f| {
        module.identifier_at(f.name).as_str() == "to_bytes" && f.module == ModuleHandleIndex(4)
    });

    if let Some(pos) = f {
        used_functions.insert(FunctionHandleIndex(pos as TableIndex));
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
