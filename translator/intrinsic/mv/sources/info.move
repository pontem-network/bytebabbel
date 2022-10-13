module self::info {
    use aptos_framework::block;
    use aptos_framework::coin;
    use aptos_framework::aptos_coin::AptosCoin;
    use aptos_framework::account;

    use self::u256::{U256, from_u64, to_address};


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

    fun gas(): U256 {
        // @todo
        from_u64(100)
    }

    fun gas_price(): U256 {
        // @todo It needs to be replaced as soon as it becomes possible to get the cost from Aptos
        from_u64(100)
    }

    fun gas_limit(): U256 {
        // @todo It needs to be replaced as soon as it becomes possible to get the cost from Aptos
        from_u64(10000000)
    }

    fun block_timestamp(): U256 {
        from_u64(block::get_epoch_interval_secs())
    }

    fun block_height(): U256 {
        from_u64(block::get_current_block_height())
    }

    fun block_hash(): U256 {
        // @todo
        from_u64(0)
    }

    fun block_difficulty(): U256 {
        // @todo
        from_u64(0)
    }

    fun block_coinbase(): U256 {
        // @todo
        from_u64(0)
    }
}