module self::persist {
    use self::u256::{U256, new_u256, from_bytes, get, zero, as_u64, from_u128, split_u128};

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
    public fun new_mem(limit: u64): Memory {
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
    public fun request_buffer_len(data: &vector<u8>): U256 {
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
    public fun effective_len(self: &mut Memory): U256 {
        from_u128((self.effective_len as u128))
    }

    // API
    public fun mload(mem: &mut Memory, offset: U256): U256 {
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
    public fun mstore(mem: &mut Memory, position: U256, value: U256) {
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
    public fun mstore8(mem: &mut Memory, position: U256, value: U256) {
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
    public fun hash(mem: &mut Memory, position: U256, length: U256): U256 {
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

    // Storage.
    //=================================================================================================================
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
