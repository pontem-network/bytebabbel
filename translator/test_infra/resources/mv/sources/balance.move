module self::balance {
    #[test_only]
    use aptos_framework::coin;

    #[test_only]
    use aptos_framework::aptos_account::{create_account};

    #[test_only]
    entry fun x42_1_000_000(core: signer){
        let (burn_cap, mint_cap) =
            aptos_framework::aptos_coin::initialize_for_test(&core);

        create_account(@test_account);
        coin::deposit(@test_account, coin::mint(1000000, &mint_cap));

        coin::destroy_burn_cap(burn_cap);
        coin::destroy_mint_cap(mint_cap);
    }

    /// AptosCoin - balance
    entry fun balance(account: address): u64 {
        use aptos_framework::aptos_coin::AptosCoin;
        use aptos_framework::coin;

        coin::balance<AptosCoin>(account)
    }
}