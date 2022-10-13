module self::helper {
    use aptos_framework::aptos_coin::AptosCoin;
    use aptos_framework::coin;
    use aptos_framework::block;

    #[test_only]
    use aptos_framework::aptos_account::{create_account};

    #[test_only]
    entry fun genesis_inic(_: signer) {
        use aptos_framework::genesis;
        genesis::setup();
    }

    #[test_only]
    entry fun fake_block(core: signer) {
        block::emit_writeset_block_event(&core, @0x990);
        block::emit_writeset_block_event(&core, @0x991);
        block::emit_writeset_block_event(&core, @0x992);
        block::emit_writeset_block_event(&core, @0x993);
    }

    entry fun block_height(_:signer): u64 {
        block::get_current_block_height()
    }

    #[test_only]
    entry fun x42_1_000_000(core: signer) {
        let (burn_cap, mint_cap) =
            aptos_framework::aptos_coin::initialize_for_test(&core);

        create_account(@test_account);
        coin::deposit(@test_account, coin::mint(1000000, &mint_cap));

        coin::destroy_burn_cap(burn_cap);
        coin::destroy_mint_cap(mint_cap);
    }


    /// AptosCoin - balance
    entry fun balance(account: address): u64 {
        coin::balance<AptosCoin>(account)
    }
}