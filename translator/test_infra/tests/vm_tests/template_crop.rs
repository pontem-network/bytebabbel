use crate::testssol::{make_move_module, sol_path};
use eth::compile::build_sol;
use eth::Flags;
use intrinsic::TEMPLATE_MODULE;
use move_binary_format::{CompiledModule, IndexKind};
use test_infra::init_log;

#[test]
pub fn test_template_crop() {
    init_log();
    let evm = build_sol(sol_path().join("mod/add.sol")).unwrap();
    let bytecode = make_move_module(
        &format!("0x42::{}", evm.name()),
        evm.contract().bin(),
        "",
        evm.contract().abi(),
        Flags::default(),
    )
    .unwrap();

    let empty_module = CompiledModule::deserialize(TEMPLATE_MODULE).unwrap();
    let module = CompiledModule::deserialize(&bytecode).unwrap();

    println!("default template vs module");

    let skip_indexes = [
        IndexKind::LocalPool,
        IndexKind::CodeDefinition,
        IndexKind::FieldDefinition,
        IndexKind::TypeParameter,
        IndexKind::MemberCount,
    ];
    for index in IndexKind::variants() {
        if skip_indexes.contains(index) {
            continue;
        }
        println!("{:?}", index);
        println!(
            "{:?} vs {:?}",
            empty_module.kind_count(*index),
            module.kind_count(*index)
        );
    }
    assert!(
        module.kind_count(IndexKind::FunctionHandle)
            < empty_module.kind_count(IndexKind::FunctionHandle),
        "completed module has more FunctionHandles than default template"
    );
}
