use move_binary_format::access::ModuleAccess;
use move_binary_format::CompiledModule;
use move_core_types::language_storage::{ModuleId, CORE_CODE_ADDRESS};
use move_core_types::resolver::MoveResolver;
use move_vm_runtime::session::Session;
use move_vm_types::gas::UnmeteredGasMeter;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn publish_std<S: MoveResolver>(session: &mut Session<'_, '_, S>) {
    publish(session, "MoveStdlib");
    publish(session, "AptosStdlib");
    publish(session, "AptosFramework");
}

fn publish<S: MoveResolver>(session: &mut Session<'_, '_, S>, dir: &str) {
    let translator_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let aptos_fw_dir = translator_dir
        .join("..")
        .join("intrinsic/mv/build/intrinsic/bytecode_modules/dependencies")
        .join(dir);

    session
        .publish_module_bundle_relax_compatibility(
            read_dir(&aptos_fw_dir),
            CORE_CODE_ADDRESS,
            &mut UnmeteredGasMeter,
        )
        .unwrap();
}

fn read_dir(dir: &Path) -> Vec<Vec<u8>> {
    let mut map = dir
        .read_dir()
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|e| e.is_file())
        .map(|p| fs::read(p).unwrap())
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
