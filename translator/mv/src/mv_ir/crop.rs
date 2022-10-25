use anyhow::Error;
use move_binary_format::{
    access::ModuleAccess,
    file_format::Visibility,
    file_format::{Bytecode, FunctionInstantiationIndex},
    file_format::{FunctionHandleIndex, ModuleHandleIndex, TableIndex},
    CompiledModule,
};

use move_binary_format::internals::ModuleIndex;
use std::collections::{HashMap, HashSet, VecDeque};

use intrinsic::{table::Info, Function};

// TODO change VecDeque to another structure
pub fn find_all_functions(module: &CompiledModule) -> Result<HashSet<FunctionHandleIndex>, Error> {
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
        let f = queue.pop_front().unwrap();
        let res = find_functions(module, f);

        for el in res.unwrap() {
            if used_functions.insert(el) {
                queue.push_back(el);
            }
        }

        if queue.is_empty() {
            break;
        }
    }

    // insert "to_bytes"
    // there's a few handles "to_bytes"
    // we need handle with ModuleHandleIndex(4)
    // TODO: delete constant this constant
    let f = module.function_handles.iter().position(|f| {
        module.identifier_at(f.name).as_str() == "to_bytes" && f.module == ModuleHandleIndex(4)
    });

    if let Some(pos) = f {
        used_functions.insert(FunctionHandleIndex(pos as TableIndex));
    }

    Ok(used_functions)
}

fn find_functions(
    module: &CompiledModule,
    func_id: FunctionHandleIndex,
) -> Result<Vec<FunctionHandleIndex>, Error> {
    // TODO: change Vec to HashSet
    let mut iter_all_functions = module.function_defs.iter();
    let mut set: Vec<FunctionHandleIndex> = vec![];

    let main_f_def = iter_all_functions.find(|&function_index| function_index.function == func_id);

    if main_f_def.is_none() {
        // here function_instations in use and
        // address_of, exists_at, create_account
        return Ok(vec![]);
    }

    let main_f_def = main_f_def.unwrap();

    for code_unit in &main_f_def.code {
        for code in &code_unit.code {
            if let Bytecode::Call(function_index) = code {
                if set.contains(function_index) {
                    continue;
                }
                set.push(*function_index);
            } else if let Bytecode::CallGeneric(function_inst) = code {
                let el = &module.function_instantiation_at(*function_inst).handle;
                if set.contains(el) {
                    continue;
                }
                set.push(*el);
            }
        }
    }

    Ok(set)
}

pub fn remove_function(
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
        def.function = *index_transaction.get(&def.function).unwrap();

        if let Some(ref mut code_unit) = def.code {
            for code in code_unit.code.iter_mut() {
                if let Bytecode::Call(func_handle) = code {
                    let new_handle = *index_transaction.get(func_handle).unwrap();
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
        inst.handle = *index_transaction.get(&inst.handle).unwrap();
    }

    // change CallGeneric in function defs
    for def in module.function_defs.iter_mut() {
        if let Some(ref mut code_unit) = def.code {
            for code in code_unit.code.iter_mut() {
                if let Bytecode::CallGeneric(func_insta_index) = code {
                    let new_index = *insta_index_transaction.get(func_insta_index).unwrap();
                    *code = Bytecode::CallGeneric(new_index);
                }
            }
        }
    }

    // signatures

    Ok(())
}
