module self::store {
    use aptos_framework::table;
    use std::signer;

    //TODO Add ability to revert state.
    struct Persist has store, key {
        tbl: table::Table<u128, u128>,
    }

    public fun init_store(self: &signer) {
        let addr = signer::borrow_address(self);
        assert!(addr == &@self, 1);
        assert!(!exists<Persist>(@self), 1);

        let store = Persist { tbl: table::new() };
        move_to(self, store);
    }

    public fun sstore(store: &mut Persist, key: u128, val: u128) {
        if (table::contains(&mut store.tbl, key)) {
            table::remove(&mut store.tbl, key);
        };

        table::add(&mut store.tbl, key, val);
    }

    public fun sload(store: &Persist, key: u128): u128 {
        if (table::contains(&store.tbl, key)) {
            *table::borrow(&store.tbl, key)
        } else {
            0
        }
    }

    #[test]
    #[expected_failure]
    public fun use_before_init() acquires Persist {
        let persist = borrow_global_mut<Persist>(@self);
        sstore(persist, 1, 1);
    }

    #[test(owner = @0x42)]
    public fun load_store_test(owner: &signer) acquires Persist {
        init_store(owner);
        let persist = borrow_global_mut<Persist>(@self);
        assert!(sload(persist, 1) == 0, 0);
        sstore(persist, 1, 1);
        assert!(sload(persist, 1) == 1, 0);
    }
}