use crate::testssol::env::executor::MoveExecutor;
use crate::testssol::{make_move_module, sol_path};
use anyhow::anyhow;
use eth::compile::build_sol;
use eth::Flags;
use move_binary_format::file_format::Bytecode;
use move_binary_format::CompiledModule;
use move_bytecode_source_map::mapping::SourceMapping;
use move_deps::move_bytecode_verifier::{CodeUnitVerifier, VerifierConfig};
use move_deps::move_package::compilation::package_layout::CompiledPackageLayout::CompiledModules;
use move_ir_types::location::Spanned;
use mv::mv_ir::print_move_module;
use test_infra::init_log;

#[allow(dead_code)]
mod testssol;

#[test]
pub fn test_loop() {
    // let mut module = CompiledModule::deserialize(include_bytes!(
    //     "../../../test_mv/build/test/bytecode_modules/Test.mv"
    // ))
    // .unwrap();
    //
    // let def = module.function_defs.get_mut(0).unwrap();
    // let code = &mut def.code.as_mut().unwrap().code;
    // code.insert(0, Bytecode::BrTrue(9));
    // code.insert(0, Bytecode::LdTrue);
    // code.insert(10, Bytecode::Branch(2));
    //
    // println!("code: {:#?}", code);
    //
    // print_move_module(&module);
    // CodeUnitVerifier::verify_module(&VerifierConfig::default(), &module)
    //     .map_err(|err| {
    //         anyhow!(
    //             "Verification error:{:?}-{:?}. Message:{:?}. Location: {:?} -{:?}",
    //             err.major_status(),
    //             err.sub_status(),
    //             err.message(),
    //             err.location(),
    //             err.indices()
    //         )
    //     })
    //     .unwrap();

    init_log();
    let evm = build_sol(sol_path().join("bitwise/simple.sol")).unwrap();
    let bytecode = make_move_module(
        &format!("0x42::{}", evm.name()),
        evm.contract().bin(),
        "",
        evm.contract().abi(),
        Flags::default(),
    )
    .unwrap();
    let mut vm = MoveExecutor::new(
        serde_json::from_str(evm.contract().abi()).unwrap(),
        Flags::default(),
    );
    vm.deploy("0x42", bytecode);

    vm.run("0x42::Simple::constructor", "0x42", None).unwrap();
    let res = vm
        .run("0x42::Simple::and_uint", "0x42", Some("10,10"))
        .unwrap()
        .to_result_str();
    assert_eq!("Uint(10)", res);
}
