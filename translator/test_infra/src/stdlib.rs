use include_dir::{include_dir, Dir};
use move_binary_format::access::ModuleAccess;
use move_binary_format::CompiledModule;
use move_core_types::language_storage::{ModuleId, CORE_CODE_ADDRESS};
use move_core_types::resolver::MoveResolver;
use move_vm_runtime::session::Session;
use move_vm_types::gas::UnmeteredGasMeter;
use std::collections::BTreeMap;

const MOVE_STD: Dir<'_> = include_dir!(
    "translator/intrinsic/mv/build/intrinsic/bytecode_modules/dependencies/MoveStdlib"
);
const APTOS_STD: Dir<'_> = include_dir!(
    "translator/intrinsic/mv/build/intrinsic/bytecode_modules/dependencies/AptosStdlib"
);
const APTOS_FW: Dir<'_> = include_dir!(
    "translator/intrinsic/mv/build/intrinsic/bytecode_modules/dependencies/AptosFramework"
);

pub fn publish_std<S: MoveResolver>(session: &mut Session<'_, '_, S>) {
    session
        .publish_module_bundle_relax_compatibility(
            sorted_code_and_modules(&MOVE_STD),
            CORE_CODE_ADDRESS,
            &mut UnmeteredGasMeter,
        )
        .unwrap();

    session
        .publish_module_bundle_relax_compatibility(
            sorted_code_and_modules(&APTOS_STD),
            CORE_CODE_ADDRESS,
            &mut UnmeteredGasMeter,
        )
        .unwrap();

    session
        .publish_module_bundle_relax_compatibility(
            sorted_code_and_modules(&APTOS_FW),
            CORE_CODE_ADDRESS,
            &mut UnmeteredGasMeter,
        )
        .unwrap();
}

pub fn sorted_code_and_modules(dir: &Dir<'_>) -> Vec<Vec<u8>> {
    let mut map = dir
        .files()
        .map(|c| c.contents().to_vec())
        .map(|c| {
            let m = CompiledModule::deserialize(&c).unwrap();
            (m.self_id(), (c, m))
        })
        .collect::<BTreeMap<_, _>>();
    let mut order = vec![];
    for id in map.keys() {
        sort_by_deps(&map, &mut order, id.clone());
    }
    let mut result = vec![];
    for id in order {
        let (code, _) = map.remove(&id).unwrap();
        result.push(code)
    }
    result
}

fn sort_by_deps(
    map: &BTreeMap<ModuleId, (Vec<u8>, CompiledModule)>,
    order: &mut Vec<ModuleId>,
    id: ModuleId,
) {
    if order.contains(&id) {
        return;
    }
    let compiled = &map.get(&id).unwrap().1;
    for dep in compiled.immediate_dependencies() {
        if map.contains_key(&dep) {
            sort_by_deps(map, order, dep);
        }
    }
    order.push(id)
}
