module self::contruct_template {
    use std::vector;

    const ELENGTH: u64 = 0x1;
    const OUT_OF_MEMORY: u64 = 0x2;
    const INVALID_RANGE: u64 = 0x3;

    const MAX_SIZE: u64 = 16 * 1024;

    // todo replace with 32 bit
    const WORD_SIZE: u64 = 16;

    struct Memory has copy, drop, store {
        data: vector<u8>,
        effective_len: u128,
        limit: u64,
    }

    public fun new(limit: u64): Memory {
        assert!(limit > 0, ELENGTH);
        assert!(limit < MAX_SIZE, ELENGTH);
        let data = vector::empty();

        Memory {
            data,
            effective_len: 0,
            limit,
        }
    }

    public fun effective_len(self: &Memory): u128 {
        self.effective_len
    }

    public fun mload(mem: &mut Memory, offset: u128): u128 {
        resize_offset(mem, offset, WORD_SIZE);
        let result = 0;

        let position = (offset as u64);

        let offset = 0u64;
        let data_len = vector::length(&mem.data);

        while (offset < WORD_SIZE) {
            let global_offset = position + offset;
            if (global_offset >= data_len) {
                break
            };
            let byte = (*vector::borrow(&mem.data, global_offset) as u128);
            let shift = (((WORD_SIZE -1 - offset) * 8) as u8);
            result = result | byte << shift;
            offset = offset + 1;
        };

        return result
    }

    public fun mstore(mem: &mut Memory, position: u128, value: u128): u128 {
        resize_offset(mem, position, WORD_SIZE);
        let position = (position as u64);
        assert!(position + WORD_SIZE < mem.limit, OUT_OF_MEMORY);

        let data_len = vector::length(&mem.data);
        while (data_len < ((position + WORD_SIZE) as u64)) {
            vector::push_back(&mut mem.data, 0);
            data_len = data_len + 1;
        };

        let offset = 0u64;
        while (offset < WORD_SIZE) {
            let shift = ((offset * 8) as u8);
            let shift = value >> shift;
            let byte = ((shift & 0xff) as u8);
            *vector::borrow_mut(&mut mem.data, position + WORD_SIZE - 1 - offset) = byte;
            offset = offset + 1;
        };
        return value
    }

    public fun mstore8(mem: &mut Memory, position: u128, value: u128) {
        resize_offset(mem, position, 1);
        let position = (position as u64);

        let value = ((value & 0xff) as u8);

        let data_len = vector::length(&mem.data);
        while (data_len < ((position + 1) as u64)) {
            vector::push_back(&mut mem.data, 0);
            data_len = data_len + 1;
        };

        *vector::borrow_mut(&mut mem.data, position) = value;
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
