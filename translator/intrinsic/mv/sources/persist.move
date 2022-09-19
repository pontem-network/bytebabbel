module self::persist {
    use self::u256::{U256, new_u256, from_bytes, get, zero, as_u64, from_u128, split_u128};

    #[test_only]
    use self::u256::{as_u128, eq};

    // Memory.
    //=================================================================================================================
    const ELENGTH: u64 = 0x1;
    const OUT_OF_MEMORY: u64 = 0x2;
    const INVALID_RANGE: u64 = 0x3;

    const WORD_SIZE: u64 = 32;

    struct Memory has copy, drop, store {
        data: vector<u8>,
        effective_len: u64,
        limit: u64,
    }

    // API
    fun new_mem(limit: u64): Memory {
        assert!(limit > 0, ELENGTH);
        let data = std::vector::empty();

        Memory {
            data,
            effective_len: 0,
            limit,
        }
    }

    /// API
    /// Returnts len of the buffer in bytes plus 4.
    fun request_buffer_len(data: &vector<u8>): U256 {
        let len = std::vector::length(data) + 4;
        new_u256(len, 0, 0, 0)
    }

    //API
    fun read_request_buffer(data: &vector<u8>, offset: U256): U256 {
        from_bytes(data, new_u256(
            get(&offset, 0) - 4, 0, 0, 0
        ))
    }

    // API
    fun effective_len(self: &mut Memory): U256 {
        from_u128((self.effective_len as u128))
    }

    // API
    fun mload(mem: &mut Memory, offset: U256): U256 {
        let position = as_u64(offset);
        resize_offset(mem, position, WORD_SIZE);

        let data_len = std::vector::length(&mem.data);

        let (v3, v2) = split_u128(mload_u128(mem, position, data_len));
        let (v1, v0) = split_u128(mload_u128(mem, position + 16, data_len));

        new_u256(v0, v1, v2, v3)
    }

    fun mload_u128(mem: &mut Memory, position: u64, data_len: u64): u128 {
        let result = 0;
        let offset = 0u64;
        while (offset < 16) {
            let global_offset = position + offset;
            if (global_offset >= data_len) {
                break
            };
            let byte = (*std::vector::borrow(&mem.data, global_offset) as u128);
            let shift = (((15 - offset) * 8) as u8);
            result = result | byte << shift;
            offset = offset + 1;
        };

        return result
    }

    // API
    fun mstore(mem: &mut Memory, position: U256, value: U256) {
        let position = as_u64(position);
        resize_offset(mem, position, WORD_SIZE);
        assert!(position + WORD_SIZE < mem.limit, OUT_OF_MEMORY);

        let data_len = std::vector::length(&mem.data);

        if (data_len < position) {
            let diff = position - data_len;
            let i = 0;
            while (i < diff) {
                std::vector::push_back(&mut mem.data, 0);
                i = i + 1;
            }
        };


        let byte_offset = 0u64;
        let word = 4;
        while (word > 0) {
            word = word - 1;
            let w = get(&value, word);
            let byte = 0u64;
            while (byte < 8) {
                let shift = (((7 - byte) * 8) as u8);
                let val = (((w >> shift) & 0xFF) as u8);
                let global_offset = position + byte_offset;
                if (global_offset >= data_len) {
                    std::vector::push_back(&mut mem.data, val);
                } else {
                    *std::vector::borrow_mut(&mut mem.data, global_offset) = val;
                };
                byte = byte + 1;
                byte_offset = byte_offset + 1;
            };
        };
    }

    // API
    fun mstore8(mem: &mut Memory, position: U256, value: U256) {
        let position = as_u64(position);
        resize_offset(mem, position, 1);

        let value = ((get(&value, 0) & 0xff) as u8);

        let data_len = std::vector::length(&mem.data);
        while (data_len < ((position + 1) as u64)) {
            std::vector::push_back(&mut mem.data, 0);
            data_len = data_len + 1;
        };

        *std::vector::borrow_mut(&mut mem.data, position) = value;
    }

    // API
    fun mslice(mem: &mut Memory, position: U256, length: U256): vector<u8> {
        let length = as_u64(length);
        if (length == 0) {
            return std::vector::empty()
        };

        let position = as_u64(position);
        let data_len = std::vector::length(&mem.data);

        let offset = 0u64;
        let slice = std::vector::empty();
        while (offset < length) {
            let global_offset = position + offset;
            if (global_offset >= data_len) {
                std::vector::push_back(&mut slice, 0);
            } else {
                std::vector::push_back(&mut slice, *std::vector::borrow(&mem.data, global_offset));
            };
            offset = offset + 1;
        };

        slice
    }

    // API
    fun hash(mem: &mut Memory, position: U256, length: U256): U256 {
        let slice = mslice(mem, position, length);
        let res = std::hash::sha3_256(slice);
        from_bytes(&res, zero())
    }

    fun resize_offset(mem: &mut Memory, offset: u64, len: u64) {
        if (len == 0) {
            return
        };

        let end = offset + len;

        if (end > mem.effective_len) {
            if (end % WORD_SIZE == 0) {
                mem.effective_len = end;
            } else {
                mem.effective_len = end + (WORD_SIZE - (end % WORD_SIZE));
            };
        };
        return
    }

    #[test]
    fun test_buff_len() {
        let buff = std::vector::empty();
        std::vector::push_back(&mut buff, 1);
        std::vector::push_back(&mut buff, 2);
        std::vector::push_back(&mut buff, 3);
        std::vector::push_back(&mut buff, 4);
        std::vector::push_back(&mut buff, 5);
        std::vector::push_back(&mut buff, 6);
        std::vector::push_back(&mut buff, 7);
        let len = as_u128(request_buffer_len(&buff));
        assert!(len == 7 + 4, 1);
    }

    #[test]
    fun test_random_access() {
        let memory = new_mem(1024);
        mstore(&mut memory, from_u128(64), from_u128(128));
        let value = mload(&mut memory, from_u128(64));
        let value = as_u128(value);
        assert!(value == 128, 1);
        let value = mload(&mut memory, from_u128(0));
        let value = as_u128(value);
        assert!(value == 0, 2);

        let value = mload(&mut memory, from_u128(32));
        let value = as_u128(value);
        assert!(value == 0, 2);
    }

    #[test]
    fun load_store_with_same_offset() {
        let memory = new_mem(1024);

        mstore(&mut memory, from_u128(0), from_u128(0x42));
        let val = mload(&mut memory, from_u128(0));
        assert!(as_u128(val) == 0x42, (as_u128(val) as u64));
        assert!(as_u128(effective_len(&mut memory)) == 32, 2);


        mstore(&mut memory, from_u128(1), from_u128(340282366920938463463374607431768211455));
        let val = mload(&mut memory, from_u128(1));
        assert!(val == from_u128(340282366920938463463374607431768211455), 1);
        assert!(as_u128(effective_len(&mut memory)) == 64, 2);
    }

    #[test]
    fun load_store_loop() {
        let memory = new_mem(2048);

        let offset = 0;
        while (offset < 1024) {
            mstore(&mut memory, from_u128(offset), from_u128(offset));
            offset = offset + 32;
        };

        let offset = 0;
        while (offset < 1024) {
            let val = mload(&mut memory, from_u128(offset));
            assert!(as_u128(val) == offset, 1);
            offset = offset + 32;
        };
    }

    #[test]
    fun mem_shift() {
        let memory = new_mem(1024);
        mstore(&mut memory, from_u128(0), from_u128(0x0000000000000000FFFFFFFFFFFFFFFF));
        let val = mload(&mut memory, from_u128(8));
        assert!(as_u128(val) == 0xFFFFFFFFFFFFFFFF0000000000000000, 0);
    }

    #[test_only]
    fun read_signer(addr: &signer): U256 {
        let encoded = std::bcs::to_bytes(addr);
        from_bytes(&encoded, zero())
    }

    #[test(
        v1 = @0xAA00000000000000000000000000000000000000000000000000000000000000,
        v2 = @0xAAFF000000000000000000000000000000000000000000000000000000000000,
        v3 = @0xAAFF110000000000000000000000000000000000000000000000000000000000,
        v4 = @0xAAFF110000000022110000000000000000000000000000000000000000000000,
        v5 = @0xAAFF110000000022110000000000003344000000000000556600000000000077,
    )]
    fun mem_store_8(v1: &signer, v2: &signer, v3: &signer, v4: &signer, v5: &signer) {
        let memory = new_mem(1024);

        mstore8(&mut memory, from_u128(0), from_u128(0xAA));
        let val = mload(&mut memory, from_u128(0));
        let expected = read_signer(v1);
        assert!(eq(val, expected), 1);

        let val = mload(&mut memory, from_u128(1));
        assert!(eq(val, zero()), 1);

        mstore8(&mut memory, from_u128(1), from_u128(0xFF));
        let val = mload(&mut memory, from_u128(0));
        let expected = read_signer(v2);
        assert!(eq(val, expected), 2);

        mstore8(&mut memory, from_u128(2), from_u128(0x11));
        let val = mload(&mut memory, from_u128(0));
        let expected = read_signer(v3);
        assert!(eq(val, expected), 3);

        mstore8(&mut memory, from_u128(8), from_u128(0x11));
        mstore8(&mut memory, from_u128(7), from_u128(0x22));
        let val = mload(&mut memory, from_u128(0));
        let expected = read_signer(v4);
        assert!(eq(val, expected), 4);

        mstore8(&mut memory, from_u128(15), from_u128(0x33));
        mstore8(&mut memory, from_u128(16), from_u128(0x44));
        mstore8(&mut memory, from_u128(23), from_u128(0x55));
        mstore8(&mut memory, from_u128(24), from_u128(0x66));
        mstore8(&mut memory, from_u128(31), from_u128(0x77));

        let val = mload(&mut memory, from_u128(0));
        let expected = read_signer(v5);
        assert!(eq(val, expected), 5);
    }

    #[test(
        s1 = @0x6261be5de65349dedcf98dad3041f331b4a397546079ef17542df4fbbf359787,
        s2 = @0x81dbecc6aee62dfb2250aeca0ca406d6dc61788004aaf116fa9c2c61d00a5897,
        hash_1 = @0xb039179a8a4ce2c252aa6f2f25798251c19b75fc1508d9d511a191e0487d64a7,
        hash_2 = @0xcc636d0dc01c94023106c1459b926e17d25572e9cbf154ec7489c6264e83ec7d,
        hash_3 = @0x1cdfbfb2fdbcc015c6c45e292d5927b775f9d6595a0c9d3ff9c029e1fe2ff7f3,
        hash_4 = @0x5129046912a39ba87d481c3c8d8cd626cbfba7f089c9879d853d40ab63ab3775,
        hash_5 = @0xc1545e05e6777d834652396ad104e7e971a78d084a9b9df34f7a16fd493bf2b0,
    )]
    fun test_hash(s1: &signer, s2: &signer, hash_1: &signer, hash_2: &signer, hash_3: &signer, hash_4: &signer, hash_5: &signer) {
        let memory = new_mem(1024);

        mstore(&mut memory, from_u128(0), read_signer(s1));
        mstore(&mut memory, from_u128(32), read_signer(s2));

        let resp = hash(&mut memory, from_u128(0), from_u128(1));
        assert!(eq(resp, read_signer(hash_1)), 1);

        let resp = hash(&mut memory, from_u128(0), from_u128(32));
        assert!(eq(resp, read_signer(hash_2)), 2);

        let resp = hash(&mut memory, from_u128(0), from_u128(64));
        assert!(eq(resp, read_signer(hash_3)), 3);

        let resp = hash(&mut memory, from_u128(0), from_u128(70));
        assert!(eq(resp, read_signer(hash_4)), 4);

        let resp = hash(&mut memory, from_u128(64), from_u128(6));
        assert!(eq(resp, read_signer(hash_5)), 5);
    }

    #[test]
    #[expected_failure(abort_code = 1)]
    fun init_zero_size() {
        new_mem(0);
    }

    // Storage.
    //=================================================================================================================
    struct Persist has store, key {
        tbl: aptos_std::table::Table<U256, U256>,
        events: aptos_std::event::EventHandle<Event>,
    }

    // API
    fun init_contract(self: &signer) {
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
    fun sstore(store: &mut Persist, key: U256, val: U256) {
        if (aptos_std::table::contains(&mut store.tbl, key)) {
            aptos_std::table::remove(&mut store.tbl, key);
        };

        aptos_std::table::add(&mut store.tbl, key, val);
    }

    // API
    fun sload(store: &mut Persist, key: U256): U256 {
        if (aptos_std::table::contains(&store.tbl, key)) {
            *aptos_std::table::borrow(&store.tbl, key)
        } else {
            zero()
        }
    }

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
