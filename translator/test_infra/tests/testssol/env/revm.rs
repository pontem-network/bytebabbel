use std::collections::BTreeMap;
use std::rc::Rc;

use anyhow::{bail, Error, Result};
use evm::backend::{MemoryBackend, MemoryVicinity};
use evm::executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata};
use evm::{Config, Context, ExitReason, Runtime};
use primitive_types::{H160, U256};

use eth::compile::EvmPack;

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
        // Execution address.
        address: "5cbdd86a2fa8dc4bddd8a8f69dba48572eec07fb".parse()?,
        // The calling EVM.
        caller: "5cbdd86a2fa8dc4bddd8a8f69dba48572eec07fb".parse()?,
        // The apparent value of the EVM. call value, if non-zero, must have payable
        apparent_value: U256::from(0u8),
    })
}

pub struct REvm {
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

    pub fn construct(&mut self, code: Vec<u8>) -> Result<()> {
        let res = self.run_tx(code)?;
        self.code = Rc::new(res);
        Ok(())
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
            ExitReason::Revert(status) => {
                bail!("ExitReason::Revert  {status:?}")
            }
            ExitReason::Succeed(status) => {
                log::trace!("ExitReason::Succeed {status:?}");
                Ok(rt.machine().return_value())
            }
        }
    }
}

impl TryFrom<Vec<u8>> for REvm {
    type Error = Error;

    fn try_from(code: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        let mut evm = REvm::new()?;
        evm.set_code(code);
        Ok(evm)
    }
}

impl TryFrom<&EvmPack> for REvm {
    type Error = Error;

    fn try_from(pack: &EvmPack) -> std::result::Result<Self, Self::Error> {
        let code = pack.code_evm()?;
        REvm::try_from(code)
    }
}

#[allow(unused_imports)]
#[cfg(test)]
mod test {
    use std::ops::Deref;
    use std::path::PathBuf;
    use std::sync::Mutex;

    use anyhow::{anyhow, Result};
    use ethabi::{Contract, Token};
    use evm::utils::I256;
    use itertools::Itertools;
    use lazy_static::lazy_static;
    use primitive_types::U256;

    use eth::abi::call::EthEncodeByString;
    use eth::compile::build_sol;
    use test_infra::init_log;

    use crate::testssol::env::revm::REvm;
    use crate::testssol::EvmPack;

    const TEST_SOL_FILE: &str = "sol/evm.sol";

    lazy_static! {
        static ref TESTFILE: Mutex<EvmPack> =
            Mutex::new(build_sol(PathBuf::from(TEST_SOL_FILE)).unwrap());
    }

    fn run_by(fn_name: &str, params: &[&str]) -> Result<String> {
        let sol = TESTFILE.lock().map_err(|err| anyhow!("{err}"))?;
        let abi: Contract = sol.abi()?;

        let mut vm = REvm::try_from(sol.deref())?;
        vm.construct(vec![])?;

        let fn_abi = &abi.functions_by_name(fn_name)?[0];

        let tx = fn_abi.call_by_vec_str(params)?;
        let result = vm.run_tx(tx)?;
        let result_token = fn_abi.decode_output(&result).unwrap();
        Ok(output_to_string(&result_token))
    }

    fn output_to_string(data: &[Token]) -> String {
        data.iter().map(|val| format!("{val:?}")).join(", ")
    }

    #[test]
    fn test_bool() {
        init_log();

        let output = run_by("without_params_bool", &[]).unwrap();
        assert_eq!(output, "Bool(true)");

        let output = run_by("param_bool", &["true", "true"]).unwrap();
        assert_eq!(output, "Bool(true)");
        let output = run_by("param_bool", &["true", "false"]).unwrap();
        assert_eq!(output, "Bool(false)");
    }

    #[test]
    fn test_num() {
        init_log();

        // with_uint
        assert_eq!(run_by("with_uint", &["2"]).unwrap(), "Uint(4)");
        assert_eq!(run_by("with_uint", &["11"]).unwrap(), "Uint(121)");

        // max_num_tuple
        let output = run_by("max_num_tuple", &["1", "2", "3", "4"]).unwrap();
        assert_eq!(output, "Int(2), Uint(4)");
    }

    #[test]
    fn test_array() {
        init_log();

        assert_eq!(
            run_by("array_bool_3", &[]).unwrap(),
            "FixedArray([Bool(true), Bool(true), Bool(false)])"
        );
        assert_eq!(
            run_by("array_bool_dyn", &[]).unwrap(),
            "Array([Bool(true), Bool(false), Bool(false), Bool(true)])"
        );
        assert_eq!(
            run_by("array_bool_dyn2", &[]).unwrap(),
            "Array([Array([Bool(false), Bool(true)]), Array([Bool(true)])])"
        );
        assert_eq!(
            run_by("array_bool_dyn3", &[]).unwrap(),
            "Array([Bool(true), Bool(false), Bool(false), Bool(true)]), Array([Array([Bool(false), Bool(true)]), Array([Bool(true)])])"
        );
    }

    #[test]
    fn test_bytes() {
        init_log();

        assert_eq!(
            run_by("byte_tuple", &[]).unwrap(),
            "Bytes([48, 49]), FixedBytes([49, 50, 51]), FixedArray([FixedBytes([48]), FixedBytes([49])]), Array([FixedBytes([48, 48]), FixedBytes([48, 49]), FixedBytes([48, 50])])"
        );
    }
}
