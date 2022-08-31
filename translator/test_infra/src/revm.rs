use std::collections::BTreeMap;
use std::rc::Rc;

use anyhow::{bail, Result};
use primitive_types::{H160, U256};

use evm_pack::backend::{MemoryBackend, MemoryVicinity};
use evm_pack::executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata};
use evm_pack::{Config, Context, ExitReason, Runtime};

fn memory_vicinity() -> Result<MemoryVicinity> {
    Ok(MemoryVicinity {
        block_base_fee_per_gas: U256::max_value(),
        gas_price: U256::from(1),
        origin: H160::random(),
        chain_id: U256::from(1u8),
        block_hashes: vec![
            "00000000000000001ebf88508a03865c71d452e25f4d51194196a1d22b6653dc".parse()?,
            "00000000000000010ff5414c5cfbe9eae982e8cef7eb2399a39118e1206c8247".parse()?,
        ],
        block_number: U256::from(0u8),
        block_coinbase: H160::zero(),
        block_timestamp: U256::from(1529891469u128),
        block_difficulty: U256::zero(),
        block_gas_limit: U256::zero(),
    })
}

fn context() -> Result<Context> {
    Ok(Context {
        // Адрес исполнения.
        address: "5cbdd86a2fa8dc4bddd8a8f69dba48572eec07fb".parse()?,
        // Вызывающий EVM.
        caller: "5cbdd86a2fa8dc4bddd8a8f69dba48572eec07fb".parse()?,
        // Кажущаяся ценность EVM.
        // значение вызова, если оно не равно нулю, должно иметь подлежащий оплате
        apparent_value: U256::from(0u8),
    })
}

struct REvm {
    code: Rc<Vec<u8>>,
    config: Config,
    vicinity: MemoryVicinity,
    ctx: Context,
}

impl REvm {
    fn new() -> Result<Self> {
        Ok(REvm {
            code: Rc::new(Vec::new()),
            config: Config::london(),
            vicinity: memory_vicinity()?,
            ctx: context()?,
        })
    }

    pub fn set_code(&mut self, code: Vec<u8>) -> &mut Self {
        self.code = Rc::new(code);
        self
    }

    pub fn run_tx(&self, call: Vec<u8>) -> Result<Vec<u8>> {
        let backend = MemoryBackend::new(&self.vicinity, BTreeMap::new());
        let metadata = StackSubstateMetadata::new(u64::MAX, &self.config);

        let precompiles = BTreeMap::new();
        let mut executor: StackExecutor<MemoryStackState<MemoryBackend>, BTreeMap<_, _>> =
            StackExecutor::new_with_precompiles(
                MemoryStackState::new(metadata, &backend),
                &self.config,
                &precompiles,
            );

        let mut rt = Runtime::new(
            self.code.clone(),
            Rc::new(call),
            self.ctx.clone(),
            &self.config,
        );
        let exit_reason = executor.execute(&mut rt);

        match exit_reason {
            ExitReason::Fatal(status) => {
                bail!("{status:?}")
            }
            ExitReason::Error(status) => {
                bail!("{status:?}")
            }
            ExitReason::Succeed(_) | ExitReason::Revert(_) => Ok(rt.machine().return_value()),
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use std::sync::Mutex;

    use lazy_static::lazy_static;

    use evm::abi::call::ToCall;
    use evm::abi::inc_ret_param::value::conv::ParamValueToRustType;
    use evm::abi::inc_ret_param::value::ParamValue;

    use crate::revm::REvm;
    use crate::sol::{build_sol_by_path, EvmPack};

    const TEST_SOL_FILE: &str = "sol/evm.sol";

    lazy_static! {
        static ref TESTFILE: Mutex<EvmPack> =
            Mutex::new(build_sol_by_path(&PathBuf::from(TEST_SOL_FILE)).unwrap());
    }

    #[test]
    fn test_bool() {
        let sol = TESTFILE.lock().unwrap();
        let abi = sol.abi().unwrap();
        let contract_bytes: Vec<u8> = sol.code_evm().unwrap();

        let mut vm = REvm::new().unwrap();
        vm.set_code(contract_bytes);

        // without_params_bool

        let fn_abi = abi.by_name("without_params_bool").unwrap();
        let call = fn_abi.try_call().unwrap();
        let tx = call.encode().unwrap();
        let result = call.decode_return(vm.run_tx(tx).unwrap()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].to_bool().unwrap(), true);

        // without_params_bool

        let fn_abi = abi.by_name("param_bool").unwrap();
        let mut call = fn_abi.try_call().unwrap();
        let tx = call
            .set_input(0, true)
            .unwrap()
            .set_input(1, true)
            .unwrap()
            .encode()
            .unwrap();
        let result = call.decode_return(vm.run_tx(tx).unwrap()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].to_bool().unwrap(), true);
    }

    #[test]
    fn test_num() {
        let sol = TESTFILE.lock().unwrap();
        let contract_bytes: Vec<u8> = sol.code_evm().unwrap();
        let abi = sol.abi().unwrap();

        let mut vm = REvm::new().unwrap();
        vm.set_code(contract_bytes);

        // with_uint

        let fn_abi = abi.by_name("with_uint").unwrap();
        let mut call = fn_abi.try_call().unwrap();

        let tx = call.set_input(0, 2usize).unwrap().encode().unwrap();
        let result = call.decode_return(vm.run_tx(tx).unwrap()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].to_isize().unwrap(), 4);

        let tx = call.set_input(0, 4u8).unwrap().encode().unwrap();
        let result = call.decode_return(vm.run_tx(tx).unwrap()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].to_isize().unwrap(), 16);

        // max_num_tuple

        let fn_abi = abi.by_name("max_num_tuple").unwrap();
        let mut call = fn_abi.try_call().unwrap();

        let tx = call
            .set_input(0, 1i8)
            .unwrap()
            .set_input(1, 2i16)
            .unwrap()
            .set_input(2, 3u32)
            .unwrap()
            .set_input(3, 4u64)
            .unwrap()
            .encode()
            .unwrap();
        let result = call.decode_return(vm.run_tx(tx).unwrap()).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].to_isize().unwrap(), 2);
        assert_eq!(result[1].to_usize().unwrap(), 4);
    }

    #[test]
    fn test_array() {
        let sol = TESTFILE.lock().unwrap();
        let abi = sol.abi().unwrap();
        let contract_bytes: Vec<u8> = sol.code_evm().unwrap();

        let mut vm = REvm::new().unwrap();
        vm.set_code(contract_bytes);

        // array_bool_3

        let fn_abi = abi.by_name("array_bool_3").unwrap();
        let call = fn_abi.try_call().unwrap();

        let tx = call.encode().unwrap();
        let result = call.decode_return(vm.run_tx(tx).unwrap()).unwrap();
        assert_eq!(result.len(), 1);

        assert_eq!(
            result[0],
            ParamValue::Array(vec![
                ParamValue::Bool(true),
                ParamValue::Bool(true),
                ParamValue::Bool(false)
            ])
        );

        // array_bool_dyn

        let fn_abi = abi.by_name("array_bool_dyn").unwrap();
        let call = fn_abi.try_call().unwrap();

        let tx = call.encode().unwrap();
        let result = call.decode_return(vm.run_tx(tx).unwrap()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            ParamValue::Array(vec![
                ParamValue::Bool(true),
                ParamValue::Bool(false),
                ParamValue::Bool(false),
                ParamValue::Bool(true),
            ])
        );

        // array_bool_dyn2

        let fn_abi = abi.by_name("array_bool_dyn2").unwrap();
        let call = fn_abi.try_call().unwrap();

        let tx = call.encode().unwrap();
        let result = call.decode_return(vm.run_tx(tx).unwrap()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            ParamValue::Array(vec![
                ParamValue::Array(vec![ParamValue::Bool(false), ParamValue::Bool(true),]),
                ParamValue::Array(vec![ParamValue::Bool(true),])
            ])
        );

        // array_bool_dyn3

        let fn_abi = abi.by_name("array_bool_dyn3").unwrap();
        let call = fn_abi.try_call().unwrap();

        let tx = call.encode().unwrap();
        let result = call.decode_return(vm.run_tx(tx).unwrap()).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0],
            ParamValue::Array(vec![
                ParamValue::Bool(true),
                ParamValue::Bool(false),
                ParamValue::Bool(false),
                ParamValue::Bool(true),
            ])
        );
        assert_eq!(
            result[1],
            ParamValue::Array(vec![
                ParamValue::Array(vec![ParamValue::Bool(false), ParamValue::Bool(true),]),
                ParamValue::Array(vec![ParamValue::Bool(true),])
            ])
        );
    }

    #[test]
    fn test_bytes() {
        let sol = TESTFILE.lock().unwrap();
        let abi = sol.abi().unwrap();
        let contract_bytes: Vec<u8> = sol.code_evm().unwrap();

        let mut vm = REvm::new().unwrap();
        vm.set_code(contract_bytes);

        // array_bool_3

        let fn_abi = abi.by_name("byte_tuple").unwrap();
        let call = fn_abi.try_call().unwrap();

        let tx = call.encode().unwrap();
        let result = call.decode_return(vm.run_tx(tx).unwrap()).unwrap();

        assert_eq!(
            result,
            vec![
                ParamValue::Bytes(vec![48, 49]),
                ParamValue::Byte(vec![49, 50, 51]),
                ParamValue::Array(vec![ParamValue::Byte(vec![48]), ParamValue::Byte(vec![49])]),
                ParamValue::Array(vec![
                    ParamValue::Byte(vec![48, 48]),
                    ParamValue::Byte(vec![48, 49]),
                    ParamValue::Byte(vec![48, 50]),
                ])
            ]
        );
    }
}
