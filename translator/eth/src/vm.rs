use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use anyhow::{bail, Error};
use ethabi::Contract;
use evm::backend::{MemoryBackend, MemoryVicinity};
use evm::executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata};
use evm::{Config, Context, ExitReason, Runtime};
use primitive_types::{H160, H256, U256};

use crate::abi::call::encode::EthEncodeByString;
use crate::bytecode::mir::constructor::make_constructor;
use crate::Mir;

pub fn static_initialization(
    bytecode: &str,
    abi: &Contract,
    args_str: &str,
    contract_addr: U256,
) -> Result<(Vec<u8>, Mir), Error> {
    let mut code = hex::decode(bytecode)?;
    let params = constructor_encode_params(abi, args_str)?;
    if !params.is_empty() {
        code.extend(&params[4..]);
    }
    let (contract_code, store) = run_initialization(code, contract_addr)?;
    let mir = make_constructor(store);
    Ok((contract_code, mir))
}

fn run_initialization(
    code: Vec<u8>,
    contract_addr: U256,
) -> Result<(Vec<u8>, HashMap<U256, U256>), Error> {
    let contract_addr = map_addr(contract_addr);
    let ctx = context(contract_addr);
    let cfg = Config::london();
    let vicinity = vicinity();
    let backend = MemoryBackend::new(&vicinity, BTreeMap::new());
    let metadata = StackSubstateMetadata::new(u64::MAX, &cfg);
    let precompiles = BTreeMap::new();
    let mut executor: StackExecutor<MemoryStackState<MemoryBackend>, BTreeMap<_, _>> =
        StackExecutor::new_with_precompiles(
            MemoryStackState::new(metadata, &backend),
            &cfg,
            &precompiles,
        );

    let mut rt = Runtime::new(Rc::new(code), Rc::new(vec![]), ctx, &cfg);
    let exit_reason = executor.execute(&mut rt);
    let store = &executor.state().substate.storages;
    let mut state = HashMap::new();
    for ((addr, key), storage) in store {
        if *addr == contract_addr {
            state.insert(map_num(*key), map_num(*storage));
        }
    }

    match exit_reason {
        ExitReason::Fatal(status) => {
            bail!("Static initialization arror: {status:?}")
        }
        ExitReason::Error(status) => {
            bail!("Static initialization arror: {status:?}")
        }
        ExitReason::Revert(status) => {
            bail!("Static initialization arror: Revert  {status:?}")
        }
        ExitReason::Succeed(_) => Ok((rt.machine().return_value(), state)),
    }
}

fn vicinity() -> MemoryVicinity {
    MemoryVicinity {
        block_base_fee_per_gas: U256::max_value(),
        gas_price: U256::from(1),
        origin: H160::random(),
        chain_id: U256::from(1u8),
        block_hashes: vec![
            "00000000000000001ebf88508a03865c71d452e25f4d51194196a1d22b6653dc"
                .parse()
                .unwrap(),
            "00000000000000010ff5414c5cfbe9eae982e8cef7eb2399a39118e1206c8247"
                .parse()
                .unwrap(),
        ],
        block_number: U256::from(0u8),
        block_coinbase: H160::zero(),
        block_timestamp: U256::from(1529891469u128),
        block_difficulty: U256::zero(),
        block_gas_limit: U256::zero(),
    }
}

fn constructor_encode_params(abi: &Contract, args_str: &str) -> Result<Vec<u8>, Error> {
    if let Some(constructor) = abi.constructor() {
        constructor.encode_value_by_str(args_str)
    } else {
        Ok(vec![])
    }
}

fn map_addr(addr: U256) -> H160 {
    let mut buf = [0u8; 32];
    addr.to_big_endian(&mut buf);
    H160::from(H256::from_slice(&buf))
}

pub fn map_num(num: H256) -> U256 {
    U256::from_big_endian(&num.0)
}

fn context(addr: H160) -> Context {
    Context {
        address: addr,
        caller: addr,
        apparent_value: U256::from(0u8),
    }
}
