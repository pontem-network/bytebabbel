module self::persist {
    // Storage.
    //=================================================================================================================

    use self::u256::{U256, zero};
    use self::memory::{Memory, mslice};

    #[test_only]
    use self::u256::{from_u128, as_u128};

    struct Persist has store, key {
        tbl: aptos_std::table::Table<U256, U256>,
        events: aptos_std::event::EventHandle<Event>,
    }

    // API
    public fun init_contract(self: &signer) {
        let addr = std::signer::address_of(self);
        assert!(addr == @self, 1);
        assert!(!exists<Persist>(@self), 1);

        if (!aptos_framework::account::exists_at(addr)) {
            aptos_framework::aptos_account::create_account(addr);
        };

        let store = Persist {
            tbl: aptos_std::table::new(),
            events: aptos_framework::account::new_event_handle(self),
        };
        move_to(self, store);
    }

    // API
    public fun sstore(store: &mut Persist, key: U256, val: U256) {
        if (aptos_std::table::contains(&mut store.tbl, key)) {
            aptos_std::table::remove(&mut store.tbl, key);
        };

        aptos_std::table::add(&mut store.tbl, key, val);
    }

    // API
    public fun sload(store: &mut Persist, key: U256): U256 {
        if (aptos_std::table::contains(&store.tbl, key)) {
            *aptos_std::table::borrow(&store.tbl, key)
        } else {
            zero()
        }
    }

    // Tests
    // problem with global borrowing

    #[test]
    #[expected_failure]
    fun use_before_init() acquires Persist {
        let persist = borrow_global_mut<Persist>(@self);
        sstore(persist, from_u128(1), from_u128(1));
    }

    #[test(owner = @0x42)]
    fun load_store_test(owner: &signer) acquires Persist {
        init_contract(owner);
        let persist = borrow_global_mut<Persist>(@self);
        assert!(as_u128(sload(persist, from_u128(1))) == 0, 0);
        sstore(persist, from_u128(1), from_u128(1));
        assert!(as_u128(sload(persist, from_u128(1))) == 1, 0);
    }

    // Events
    //==========================================================================
    struct Event has store, drop {
        data: vector<u8>,
        topics: vector<U256>,
    }

    // API
    fun log0(persist: &mut Persist, mem: &mut Memory, offset: U256, len: U256) {
        let data = mslice(mem, offset, len);
        let event = Event { data, topics: std::vector::empty() };
        aptos_std::event::emit_event<Event>(&mut persist.events, event);
    }

    // API
    fun log1(persist: &mut Persist, mem: &mut Memory, offset: U256, len: U256, topic: U256) {
        let data = mslice(mem, offset, len);
        let topics = std::vector::singleton(topic);
        let event = Event { data, topics };
        aptos_std::event::emit_event<Event>(&mut persist.events, event);
    }

    // API
    fun log2(persist: &mut Persist, mem: &mut Memory, offset: U256, len: U256, topic1: U256, topic2: U256) {
        let data = mslice(mem, offset, len);
        let topics = std::vector::empty();
        std::vector::push_back(&mut topics, topic1);
        std::vector::push_back(&mut topics, topic2);
        let event = Event { data, topics };
        aptos_std::event::emit_event<Event>(&mut persist.events, event);
    }

    // API
    fun log3(persist: &mut Persist, mem: &mut Memory, offset: U256, len: U256, topic1: U256, topic2: U256, topic3: U256) {
        let data = mslice(mem, offset, len);
        let topics = std::vector::empty();
        std::vector::push_back(&mut topics, topic1);
        std::vector::push_back(&mut topics, topic2);
        std::vector::push_back(&mut topics, topic3);
        let event = Event { data, topics };
        aptos_std::event::emit_event<Event>(&mut persist.events, event);
    }

    // API
    fun log4(persist: &mut Persist, mem: &mut Memory, offset: U256, len: U256, topic1: U256, topic2: U256, topic3: U256, topic4: U256) {
        let data = mslice(mem, offset, len);
        let topics = std::vector::empty();
        std::vector::push_back(&mut topics, topic1);
        std::vector::push_back(&mut topics, topic2);
        std::vector::push_back(&mut topics, topic3);
        std::vector::push_back(&mut topics, topic4);
        let event = Event { data, topics };
        aptos_std::event::emit_event<Event>(&mut persist.events, event);
    }
}
