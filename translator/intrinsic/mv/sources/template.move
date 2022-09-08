module self::template {
    // Memory.
    //=================================================================================================================
    const ELENGTH: u64 = 0x1;
    const OUT_OF_MEMORY: u64 = 0x2;
    const INVALID_RANGE: u64 = 0x3;

    // todo replace with 32 bit
    const WORD_SIZE: u64 = 16;

    struct Memory has copy, drop, store {
        data: vector<u8>,
        effective_len: u128,
        limit: u64,
    }

    fun new_mem(limit: u64): Memory {
        assert!(limit > 0, ELENGTH);
        let data = std::vector::empty();

        Memory {
            data,
            effective_len: 0,
            limit,
        }
    }

    fun effective_len(self: &mut Memory): u128 {
        self.effective_len
    }

    fun mload(mem: &mut Memory, offset: u128): u128 {
        resize_offset(mem, offset, WORD_SIZE);
        let result = 0;

        let position = (offset as u64);

        let offset = 0u64;
        let data_len = std::vector::length(&mem.data);

        while (offset < WORD_SIZE) {
            let global_offset = position + offset;
            if (global_offset >= data_len) {
                break
            };
            let byte = (*std::vector::borrow(&mem.data, global_offset) as u128);
            let shift = (((WORD_SIZE -1 - offset) * 8) as u8);
            result = result | byte << shift;
            offset = offset + 1;
        };

        return result
    }

    fun mstore(mem: &mut Memory, position: u128, value: u128) {
        resize_offset(mem, position, WORD_SIZE);
        let position = (position as u64);
        assert!(position + WORD_SIZE < mem.limit, OUT_OF_MEMORY);

        let data_len = std::vector::length(&mem.data);
        while (data_len < ((position + WORD_SIZE) as u64)) {
            std::vector::push_back(&mut mem.data, 0);
            data_len = data_len + 1;
        };

        let offset = 0u64;
        while (offset < WORD_SIZE) {
            let shift = ((offset * 8) as u8);
            let shift = value >> shift;
            let byte = ((shift & 0xff) as u8);
            *std::vector::borrow_mut(&mut mem.data, position + WORD_SIZE - 1 - offset) = byte;
            offset = offset + 1;
        };
    }

    fun mstore8(mem: &mut Memory, position: u128, value: u128) {
        resize_offset(mem, position, 1);
        let position = (position as u64);

        let value = ((value & 0xff) as u8);

        let data_len = std::vector::length(&mem.data);
        while (data_len < ((position + 1) as u64)) {
            std::vector::push_back(&mut mem.data, 0);
            data_len = data_len + 1;
        };

        *std::vector::borrow_mut(&mut mem.data, position) = value;
    }

    fun resize_offset(mem: &mut Memory, offset: u128, len: u64) {
        if (len == 0) {
            return
        };

        let end = offset + (len as u128);

        if (end > mem.effective_len) {
            mem.effective_len = next_multiple_of_word(end);
        };
        return
    }

    fun next_multiple_of_word(x: u128): u128 {
        let word_size = (WORD_SIZE as u128);
        if (x % word_size == 0) {
            return x
        };

        return x + (word_size - (x % word_size))
    }

    // Storage.
    //=================================================================================================================
    struct Persist has store, key {
        tbl: aptos_std::table::Table<u128, u128>,
    }

    fun init_store(self: &signer) {
        let addr = std::signer::borrow_address(self);
        assert!(addr == &@self, 1);
        assert!(!exists<Persist>(@self), 1);

        let store = Persist { tbl: aptos_std::table::new() };
        move_to(self, store);
    }

    fun sstore(store: &mut Persist, key: u128, val: u128) {
        if (aptos_std::table::contains(&mut store.tbl, key)) {
            aptos_std::table::remove(&mut store.tbl, key);
        };

        aptos_std::table::add(&mut store.tbl, key, val);
    }

    fun sload(store: &mut Persist, key: u128): u128 {
        if (aptos_std::table::contains(&store.tbl, key)) {
            *aptos_std::table::borrow(&store.tbl, key)
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


    #[test]
    fun load_store_with_same_offset() {
        let memory = new_mem(1024);

        mstore(&mut memory, 0, 0x42);
        let val = mload(&mut memory, 0);
        assert!(val == 0x42, 0);
        assert!(effective_len(&mut memory) == 16, 2);


        mstore(&mut memory, 1, 340282366920938463463374607431768211455);
        let val = mload(&mut memory, 1);
        assert!(val == 340282366920938463463374607431768211455, 1);
        assert!(effective_len(&mut memory) == 32, 2);
    }

    #[test]
    fun load_store_loop() {
        let memory = new_mem(2048);

        let offset = 0;
        while (offset < 1024) {
            mstore(&mut memory, offset, offset);
            offset = offset + 16;
        };

        let offset = 0;
        while (offset < 1024) {
            let val = mload(&mut memory, offset);
            assert!(val == offset, 1);
            offset = offset + 16;
        };
    }

    #[test]
    fun mem_shift() {
        let memory = new_mem(1024);
        mstore(&mut memory, 0, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
        let val = mload(&mut memory, 8);
        assert!(val == 0xFFFFFFFFFFFFFFFF0000000000000000, 0);
    }

    #[test]
    fun mem_store_8() {
        let memory = new_mem(1024);

        mstore8(&mut memory, 0, 0xAA);
        let val = mload(&mut memory, 0);
        assert!(val == 0xAA000000000000000000000000000000, 1);

        mstore8(&mut memory, 1, 0xFF);
        let val = mload(&mut memory, 0);
        assert!(val == 0xAAFF0000000000000000000000000000, 2);

        mstore8(&mut memory, 2, 0x11);
        let val = mload(&mut memory, 0);
        assert!(val == 0xAAFF1100000000000000000000000000, 3);

        mstore8(&mut memory, 15, 0xCC);
        let val = mload(&mut memory, 0);
        assert!(val == 0xAAFF11000000000000000000000000CC, (val as u64));
    }

    #[test]
    #[expected_failure(abort_code = 1)]
    fun init_zero_size() {
        new_mem(0);
    }

    // Cast
    //=================================================================================================================
    fun address_to_number(addr: &signer): u128 {
        let encoded = std::bcs::to_bytes(addr);
        let result = 0;

        let offset = 16u64;
        let len = 32u64;
        while (offset < len) {
            let byte = (*std::vector::borrow(&encoded, offset) as u128);
            let shift = (((len -1 - offset) * 8) as u8);
            result = result | byte << shift;
            offset = offset + 1;
        };

        return result
    }


    #[test(self = @0x42)]
    fun test_address_to_u128_1(self: &signer) {
        assert!(address_to_number(self) == 0x42, 0);
    }

    #[test(self = @0x4213421342134213)]
    fun test_address_to_u128_2(self: &signer) {
        assert!(address_to_number(self) == 0x4213421342134213, 0);
    }
}
