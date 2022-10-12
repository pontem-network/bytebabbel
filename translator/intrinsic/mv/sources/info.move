module self::info {
    use aptos_framework::block;
    use aptos_framework::coin;
    use aptos_framework::aptos_coin::AptosCoin;
    use aptos_framework::account;
    use self::u256::{U256, from_u64};

    #[test_only]
    use aptos_framework::aptos_account::{create_account, transfer};

    #[test_only]
    use std::signer;

    /// AptosCoin - balance
    fun balance(account: address): U256 {
        let balance;
        if( account::exists_at(account) ){
            balance = coin::balance<AptosCoin>(account);
        }else{
            balance = 0;
        };

        from_u64(balance)
    }

    fun get_current_block_height(): U256 {
        from_u64(block::get_current_block_height())
    }

    fun get_epoch_interval_secs(): U256 {
        from_u64(block::get_epoch_interval_secs())
    }

    fun gas_price():U256{
        // @todo It needs to be replaced as soon as it becomes possible to get the cost from Aptos
        from_u64(100)
    }

    fun maximum_number_of_gas_units():U256{
        // @todo It needs to be replaced as soon as it becomes possible to get the cost from Aptos
        from_u64(10000000)
    }

    #[test(core = @0x1, test_admin = @test_token_admin, test_account = @test_account)]
    public fun test_balance(core: signer, test_admin: signer, test_account: address) {
        let (burn_cap, mint_cap) = aptos_framework::aptos_coin::initialize_for_test(&core);
        create_account(signer::address_of(&test_admin));
        coin::deposit(signer::address_of(&test_admin), coin::mint(10000, &mint_cap));

        transfer(&test_admin, test_account, 500);

        assert!(coin::balance<AptosCoin>(signer::address_of(&test_admin)) == 9500, 1);
        assert!(coin::balance<AptosCoin>(test_account) == 500, 2);

        let admin_balance = balance(signer::address_of(&test_admin));
        assert!(admin_balance == from_u64(9500), 3);
        let test_account_balance = balance(test_account);
        assert!(test_account_balance == from_u64(500), 4);

        coin::destroy_burn_cap(burn_cap);
        coin::destroy_mint_cap(mint_cap);
    }
}