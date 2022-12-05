module self::info {
    use aptos_framework::block;
    use aptos_framework::coin;
    use aptos_framework::aptos_coin::AptosCoin;
    use aptos_framework::account;

    use external::u256::{from_u64, to_address};


    /// AptosCoin - balance
    fun balance(account256: U256): U256 {
        let account = to_address(account256);

        let balance;
        if (account::exists_at(account)) {
            balance = coin::balance<AptosCoin>(account);
        }else {
            balance = 0;
        };

        from_u64(balance)
    }

    // msg.gas (uint): remaining gas
    fun gas(): U256 {
        assert!(false,0);
        from_u64(0)
    }

    // gasprice() (uint): gas price of the transaction
    fun gas_price(): U256 {
        // @todo It needs to be replaced as soon as it becomes possible to get the cost from Aptos
        from_u64(100)
    }

    // gaslimit() (uint)
    fun gas_limit(): U256 {
        // @todo It needs to be replaced as soon as it becomes possible to get the cost from Aptos
        from_u64(10000000)
    }

    // block.timestamp (uint): current block timestamp
    fun block_timestamp(): U256 {
        from_u64(block::get_epoch_interval_secs())
    }

    // block.number (uint): current block number
    fun block_height(): U256 {
        from_u64(block::get_current_block_height())
    }

    // deprecated
    // block.blockhash (function(uint) returns (bytes32)): hash of the given block
    fun block_hash(num:U256): address {
        let encoded = std::bcs::to_bytes(&num);
        let address = aptos_framework::util::address_from_bytes(encoded);
        return address
    }

    // block.difficulty (uint): current block difficulty
    fun block_difficulty(): U256 {
        assert!(false,0);
        from_u64(0)
    }

    // block.coinbase (address): current block miner's address
    fun block_coinbase(): U256 {
        assert!(false,0);
        from_u64(0)
    }

}