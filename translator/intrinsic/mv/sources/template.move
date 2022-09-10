module self::template {
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
    fun bytes_len(data: &vector<u8>): U256 {
        let len = std::vector::length(data);
        U256 {
           v0: len, v1: 0, v2: 0, v3: 0
        }
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

        return U256 {
            v0,
            v1,
            v2,
            v3,
        }
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
        let position = as_u64(position);
        let length = as_u64(length);
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
        from_bytes(&res, 0)
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
        let len = as_u128(bytes_len(&buff));
        assert!(len == 7, 1);
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
        mstore(&mut memory, from_u128(0), from_u128(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));
        let val = mload(&mut memory, from_u128(8));
        assert!(as_u128(val) == 0xFFFFFFFFFFFFFFFF0000000000000000, 0);
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
        let expected = from_address(v1);
        assert!(eq(val, expected), 1);

        let val = mload(&mut memory, from_u128(1));
        assert!(eq(val, zero()), 1);

        mstore8(&mut memory, from_u128(1), from_u128(0xFF));
        let val = mload(&mut memory, from_u128(0));
        let expected = from_address(v2);
        assert!(eq(val, expected), 2);

        mstore8(&mut memory, from_u128(2), from_u128(0x11));
        let val = mload(&mut memory, from_u128(0));
        let expected = from_address(v3);
        assert!(eq(val, expected), 3);

        mstore8(&mut memory, from_u128(8), from_u128(0x11));
        mstore8(&mut memory, from_u128(7), from_u128(0x22));
        let val = mload(&mut memory, from_u128(0));
        let expected = from_address(v4);
        assert!(eq(val, expected), 4);

        mstore8(&mut memory, from_u128(15), from_u128(0x33));
        mstore8(&mut memory, from_u128(16), from_u128(0x44));
        mstore8(&mut memory, from_u128(23), from_u128(0x55));
        mstore8(&mut memory, from_u128(24), from_u128(0x66));
        mstore8(&mut memory, from_u128(31), from_u128(0x77));

        let val = mload(&mut memory, from_u128(0));
        let expected = from_address(v5);
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

        mstore(&mut memory, from_u128(0), from_address(s1));
        mstore(&mut memory, from_u128(32), from_address(s2));

        let resp = hash(&mut memory, from_u128(0), from_u128(1));
        assert!(eq(resp, from_address(hash_1)), 1);

        let resp = hash(&mut memory, from_u128(0), from_u128(32));
        assert!(eq(resp, from_address(hash_2)), 2);

        let resp = hash(&mut memory, from_u128(0), from_u128(64));
        assert!(eq(resp, from_address(hash_3)), 3);

        let resp = hash(&mut memory, from_u128(0), from_u128(70));
        assert!(eq(resp, from_address(hash_4)), 4);

        let resp = hash(&mut memory, from_u128(64), from_u128(6));
        assert!(eq(resp, from_address(hash_5)), 5);
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

    // U256
    //=================================================================================================================
    // Errors.
    /// When trying to get or put word into U256 but it's out of index.
    const EWORDS_OVERFLOW: u64 = 1;

    // Constants.

    /// Max `u64` value.
    const U64_MAX: u128 = 18446744073709551615;

    /// Max `u128` value.
    const U128_MAX: u128 = 340282366920938463463374607431768211455;

    /// Total words in `U256` (64 * 4 = 256).
    const WORDS: u64 = 4;

    /// When both `U256` equal.
    const EQUAL: u8 = 0;

    /// When `a` is less than `b`.
    const LESS_THAN: u8 = 1;

    /// When `b` is greater than `b`.
    const GREATER_THAN: u8 = 2;

    // Data structs.
    /// The `U256` resource.
    /// Contains 4 u64 numbers.
    struct U256 has copy, drop, store {
        v0: u64,
        v1: u64,
        v2: u64,
        v3: u64,
    }

    /// Double `U256` used for multiple (to store overflow).
    struct DU256 has copy, drop, store {
        v0: u64,
        v1: u64,
        v2: u64,
        v3: u64,
        v4: u64,
        v5: u64,
        v6: u64,
        v7: u64,
    }

    // API
    /// Adds two `U256` and returns sum.
    fun overflowing_add(a: U256, b: U256): U256 {
        let ret = zero();
        let carry = 0u64;

        let i = 0;
        while (i < WORDS) {
            let a1 = get(&a, i);
            let b1 = get(&b, i);

            if (carry != 0) {
                let (res1, is_overflow1) = overflowing_add_u64(a1, b1);
                let (res2, is_overflow2) = overflowing_add_u64(res1, carry);
                put(&mut ret, i, res2);

                carry = 0;
                if (is_overflow1) {
                    carry = carry + 1;
                };

                if (is_overflow2) {
                    carry = carry + 1;
                }
            } else {
                let (res, is_overflow) = overflowing_add_u64(a1, b1);
                put(&mut ret, i, res);

                carry = 0;
                if (is_overflow) {
                    carry = 1;
                };
            };

            i = i + 1;
        };
        ret
    }

    /// Convert `U256` to `u128` value if possible (otherwise it aborts).
    fun as_u128(a: U256): u128 {
        ((a.v1 as u128) << 64) + (a.v0 as u128)
    }

    fun as_u64(a: U256): u64 {
        a.v0
    }

    /// Compares two `U256` numbers.
    fun compare(a: &U256, b: &U256): u8 {
        let i = WORDS;
        while (i > 0) {
            i = i - 1;
            let a1 = get(a, i);
            let b1 = get(b, i);

            if (a1 != b1) {
                if (a1 < b1) {
                    return LESS_THAN
                } else {
                    return GREATER_THAN
                }
            }
        };

        EQUAL
    }

    /// Returns a `U256` from `u128` value.
    fun from_u128(val: u128): U256 {
        let (a2, a1) = split_u128(val);
        U256 {
            v0: a1,
            v1: a2,
            v2: 0,
            v3: 0,
        }
    }

    // API
    /// Multiples two `U256`.
    fun overflowing_mul(a: U256, b: U256): U256 {
        let ret = DU256 {
            v0: 0,
            v1: 0,
            v2: 0,
            v3: 0,
            v4: 0,
            v5: 0,
            v6: 0,
            v7: 0,
        };

        let i = 0;
        while (i < WORDS) {
            let carry = 0u64;
            let b1 = get(&b, i);

            let j = 0;
            while (j < WORDS) {
                let a1 = get(&a, j);

                if (a1 != 0 || carry != 0) {
                    let (hi, low) = split_u128((a1 as u128) * (b1 as u128));

                    let overflow = {
                        let existing_low = get_d(&ret, i + j);
                        let (low, o) = overflowing_add_u64(low, existing_low);
                        put_d(&mut ret, i + j, low);
                        if (o) {
                            1
                        } else {
                            0
                        }
                    };

                    carry = {
                        let existing_hi = get_d(&ret, i + j + 1);
                        let hi = hi + overflow;
                        let (hi, o0) = overflowing_add_u64(hi, carry);
                        let (hi, o1) = overflowing_add_u64(hi, existing_hi);
                        put_d(&mut ret, i + j + 1, hi);

                        if (o0 || o1) {
                            1
                        } else {
                            0
                        }
                    };
                };

                j = j + 1;
            };

            i = i + 1;
        };

        let (r, _overflow) = du256_to_u256(ret);
        r
    }

    // API
    /// Subtracts two `U256`, returns result.
    fun overflowing_sub(a: U256, b: U256): U256 {
        let ret = zero();

        let carry = 0u64;

        let i = 0;
        while (i < WORDS) {
            let a1 = get(&a, i);
            let b1 = get(&b, i);

            if (carry != 0) {
                let (res1, is_overflow1) = overflowing_sub_u64(a1, b1);
                let (res2, is_overflow2) = overflowing_sub_u64(res1, carry);
                put(&mut ret, i, res2);

                carry = 0;
                if (is_overflow1) {
                    carry = carry + 1;
                };

                if (is_overflow2) {
                    carry = carry + 1;
                }
            } else {
                let (res, is_overflow) = overflowing_sub_u64(a1, b1);
                put(&mut ret, i, res);

                carry = 0;
                if (is_overflow) {
                    carry = 1;
                };
            };

            i = i + 1;
        };
        ret
    }

    // API
    fun lt(a: U256, b: U256): bool {
        compare(&a, &b) == LESS_THAN
    }

    // API
    fun le(a: U256, b: U256): bool {
        compare(&a, &b) != GREATER_THAN
    }

    // API
    fun gt(a: U256, b: U256): bool {
        compare(&a, &b) == GREATER_THAN
    }

    // API
    fun ge(a: U256, b: U256): bool {
        compare(&a, &b) != LESS_THAN
    }

    // API
    fun eq(a: U256, b: U256): bool {
        compare(&a, &b) == EQUAL
    }

    // API
    fun ne(a: U256, b: U256): bool {
        compare(&a, &b) != EQUAL
    }

    // API
    fun bitnot(a: U256): U256 {
        let ret = zero();
        let i = 0;
        while (i < WORDS) {
            put(&mut ret, i, get(&a, i) ^ 0xFFFFFFFFFFFFFFFF);
            i = i + 1;
        };
        ret
    }

    // API
    fun byte(i: U256, x: U256): U256 {
        let shift = 248 - as_u128(i) * 8;
        bitand(shr_u8(x, (shift as u8)), from_u128(0xFF))
    }

    // API
    /// Divide `a` by `b`.
    fun div(a: U256, b: U256): U256 {
        let ret = zero();

        let a_bits = bits(&a);
        let b_bits = bits(&b);

        if (b_bits == 0) {
            return ret
        };

        if (a_bits < b_bits) {
            return ret
        };

        let shift = a_bits - b_bits;
        b = shl_u8(b, (shift as u8));

        loop {
            let cmp = compare(&a, &b);
            if (cmp == GREATER_THAN || cmp == EQUAL) {
                let index = shift / 64;
                let m = get(&ret, index);
                let c = m | 1 << ((shift % 64) as u8);
                put(&mut ret, index, c);

                a = overflowing_sub(a, b);
            };

            b = shr_u8(b, 1);
            if (shift == 0) {
                break
            };

            shift = shift - 1;
        };

        ret
    }


    // API
    /// Divide `a` by `b` with sign.
    /// todo check this)
    fun sdiv(a: U256, b: U256): U256 {
        let a_neg = is_negative(&a);
        let b_neg = is_negative(&b);

        let a = if (a_neg) { bitnot(a) } else { a };
        let b = if (b_neg) { bitnot(b) } else { b };

        let ret = div(a, b);

        if (a_neg != b_neg) {
            bitnot(ret)
        } else {
            ret
        }
    }

    // API
    /// Signed lt.
    /// todo check this)
    fun slt(a: U256, b: U256): bool {
        let a_neg = is_negative(&a);
        let b_neg = is_negative(&b);

        if (a_neg && !b_neg) {
            return true
        };

        if (!a_neg && b_neg) {
            return false
        };

        if (a_neg && b_neg) {
            return gt(a, b)
        };

        lt(a, b)
    }

    // API
    /// Signed gt.
    /// todo check this)
    fun sgt(a: U256, b: U256): bool {
        let a_neg = is_negative(&a);
        let b_neg = is_negative(&b);

        if (a_neg && !b_neg) {
            return false
        };

        if (!a_neg && b_neg) {
            return true
        };

        if (a_neg && b_neg) {
            return lt(a, b)
        };

        gt(a, b)
    }

    // API
    /// Signed mod.
    /// todo check this)
    fun smod(a: U256, b: U256): U256 {
        let a_neg = is_negative(&a);
        let b_neg = is_negative(&b);

        let a = if (a_neg) { bitnot(a) } else { a };
        let b = if (b_neg) { bitnot(b) } else { b };

        let ret = mod(a, b);

        if (a_neg) {
            bitnot(ret)
        } else {
            ret
        }
    }

    // API
    /// Exponentiation.
    /// todo check this)
    /// todo use DU256 for intermediate calculations
    fun exp(a: U256, b: U256): U256 {
        let ret = one();
        let i = 0;
        while (i < WORDS) {
            let b1 = get(&b, i);
            let j = 0;
            while (j < 64) {
                if ((b1 & (1 << j)) != 0) {
                    ret = overflowing_mul(ret, a);
                };
                a = overflowing_mul(a, a);
                j = j + 1;
            };
            i = i + 1;
        };
        ret
    }

    // API
    /// Signed exponentiation.
    fun sexp(a: U256, b: U256): U256 {
        // todo replace with signed exp
        exp(a, b)
    }

    // API
    /// Signed shift right.
    fun sar(a: U256, b: U256): U256 {
        // todo repalce with signed shift right
        shr(a, b)
    }

    fun one(): U256 {
        U256 {
            v0: 1,
            v1: 0,
            v2: 0,
            v3: 0,
        }
    }

    fun is_negative(a: &U256): bool {
        let msb = get(a, WORDS - 1);
        msb & 0x8000000000000000 != 0
    }

    // API
    /// Mod `a` by `b`.
    fun mod(a: U256, b: U256): U256 {
        let ret = zero();

        let a_bits = bits(&a);
        let b_bits = bits(&b);

        if (b_bits == 0) {
            return ret
        };

        if (a_bits < b_bits) {
            return a
        };

        let shift = a_bits - b_bits;
        b = shl_u8(b, (shift as u8));

        loop {
            let cmp = compare(&a, &b);
            if (cmp == GREATER_THAN || cmp == EQUAL) {
                a = overflowing_sub(a, b);
            };

            b = shr_u8(b, 1);
            if (shift == 0) {
                break
            };

            shift = shift - 1;
        };

        a
    }

    // API
    /// Binary or `a` by `b`.
    fun bitor(a: U256, b: U256): U256 {
        let ret = zero();

        let i = 0;
        while (i < WORDS) {
            let a1 = get(&a, i);
            let b1 = get(&b, i);
            put(&mut ret, i, a1 | b1);

            i = i + 1;
        };

        ret
    }

    // API
    /// Binary and `a` by `b`.
    fun bitand(a: U256, b: U256): U256 {
        let ret = zero();

        let i = 0;
        while (i < WORDS) {
            let a1 = get(&a, i);
            let b1 = get(&b, i);
            put(&mut ret, i, a1 & b1);

            i = i + 1;
        };

        ret
    }

    // API
    /// Binary xor `a` by `b`.
    fun bitxor(a: U256, b: U256): U256 {
        let ret = zero();

        let i = 0;
        while (i < WORDS) {
            let a1 = get(&a, i);
            let b1 = get(&b, i);
            put(&mut ret, i, a1 ^ b1);

            i = i + 1;
        };

        ret
    }

    /// Shift right `a`  by `shift`.
    fun shr_u8(a: U256, shift: u8): U256 {
        let ret = zero();

        let word_shift = (shift as u64) / 64;
        let bit_shift = (shift as u64) % 64;

        let i = word_shift;
        while (i < WORDS) {
            let m = get(&a, i) >> (bit_shift as u8);
            put(&mut ret, i - word_shift, m);
            i = i + 1;
        };

        if (bit_shift > 0) {
            let j = word_shift + 1;
            while (j < WORDS) {
                let m = get(&ret, j - word_shift - 1) + (get(&a, j) << (64 - (bit_shift as u8)));
                put(&mut ret, j - word_shift - 1, m);
                j = j + 1;
            };
        };

        ret
    }

    // API
    fun shr(a: U256, shift: U256): U256 {
        let ret = zero();
        let shift = as_u128(shift);

        if (is_zero(a) || shift >= 256) {
            return ret
        };

        return shr_u8(a, (shift as u8))
    }

    /// Shift left `a` by `shift`.
    fun shl_u8(a: U256, shift: u8): U256 {
        let ret = zero();

        let word_shift = (shift as u64) / 64;
        let bit_shift = (shift as u64) % 64;

        let i = word_shift;
        while (i < WORDS) {
            let m = get(&a, i - word_shift) << (bit_shift as u8);
            put(&mut ret, i, m);
            i = i + 1;
        };

        if (bit_shift > 0) {
            let j = word_shift + 1;

            while (j < WORDS) {
                let m = get(&ret, j) + (get(&a, j - 1 - word_shift) >> (64 - (bit_shift as u8)));
                put(&mut ret, j, m);
                j = j + 1;
            };
        };

        ret
    }

    // API
    fun shl(a: U256, shift: U256): U256 {
        let ret = zero();
        let shift = as_u128(shift);

        if (is_zero(a) || shift >= 256) {
            return ret
        };

        return shl_u8(a, (shift as u8))
    }

    // API
    fun is_zero(a: U256): bool {
        a.v0 == 0 && a.v1 == 0 && a.v2 == 0 && a.v3 == 0
    }

    /// Returns `U256` equals to zero.
    fun zero(): U256 {
        U256 {
            v0: 0,
            v1: 0,
            v2: 0,
            v3: 0,
        }
    }

    // Private functions.
    /// Get bits used to store `a`.
    fun bits(a: &U256): u64 {
        let i = 1;
        while (i < WORDS) {
            let a1 = get(a, WORDS - i);
            if (a1 > 0) {
                return ((0x40 * (WORDS - i + 1)) - (leading_zeros_u64(a1) as u64))
            };

            i = i + 1;
        };

        let a1 = get(a, 0);
        0x40 - (leading_zeros_u64(a1) as u64)
    }

    /// Get leading zeros of a binary representation of `a`.
    fun leading_zeros_u64(a: u64): u8 {
        if (a == 0) {
            return 64
        };

        let a1 = a & 0xFFFFFFFF;
        let a2 = a >> 32;

        if (a2 == 0) {
            let bit = 32;

            while (bit >= 1) {
                let b = (a1 >> (bit - 1)) & 1;
                if (b != 0) {
                    break
                };

                bit = bit - 1;
            };

            (32 - bit) + 32
        } else {
            let bit = 64;
            while (bit >= 1) {
                let b = (a >> (bit - 1)) & 1;
                if (b != 0) {
                    break
                };
                bit = bit - 1;
            };

            64 - bit
        }
    }

    /// Similar to Rust `overflowing_add`.
    /// Returns a tuple of the addition along with a boolean indicating whether an arithmetic overflow would occur.
    /// If an overflow would have occurred then the wrapped value is returned.
    fun overflowing_add_u64(a: u64, b: u64): (u64, bool) {
        let a128 = (a as u128);
        let b128 = (b as u128);

        let r = a128 + b128;
        if (r > U64_MAX) {
            // overflow
            let overflow = r - U64_MAX - 1;
            ((overflow as u64), true)
        } else {
            (((a128 + b128) as u64), false)
        }
    }

    /// Similar to Rust `overflowing_sub`.
    /// Returns a tuple of the addition along with a boolean indicating whether an arithmetic overflow would occur.
    /// If an overflow would have occurred then the wrapped value is returned.
    fun overflowing_sub_u64(a: u64, b: u64): (u64, bool) {
        if (a < b) {
            let r = b - a;
            ((U64_MAX as u64) - r + 1, true)
        } else {
            (a - b, false)
        }
    }

    /// Extracts two `u64` from `a` `u128`.
    fun split_u128(a: u128): (u64, u64) {
        let a1 = ((a >> 64) as u64);
        let a2 = ((a & 0xFFFFFFFFFFFFFFFF) as u64);

        (a1, a2)
    }

    /// Get word from `a` by index `i`.
    fun get(a: &U256, i: u64): u64 {
        if (i == 0) {
            a.v0
        } else if (i == 1) {
            a.v1
        } else if (i == 2) {
            a.v2
        } else if (i == 3) {
            a.v3
        } else {
            abort EWORDS_OVERFLOW
        }
    }

    /// Get word from `DU256` by index.
    fun get_d(a: & DU256, i: u64): u64 {
        if (i == 0) {
            a.v0
        } else if (i == 1) {
            a.v1
        } else if (i == 2) {
            a.v2
        } else if (i == 3) {
            a.v3
        } else if (i == 4) {
            a.v4
        } else if (i == 5) {
            a.v5
        } else if (i == 6) {
            a.v6
        } else if (i == 7) {
            a.v7
        } else {
            abort EWORDS_OVERFLOW
        }
    }

    /// Put new word `val` into `U256` by index `i`.
    fun put(a: &mut U256, i: u64, val: u64) {
        if (i == 0) {
            a.v0 = val;
        } else if (i == 1) {
            a.v1 = val;
        } else if (i == 2) {
            a.v2 = val;
        } else if (i == 3) {
            a.v3 = val;
        } else {
            abort EWORDS_OVERFLOW
        }
    }

    /// Put new word into `DU256` by index `i`.
    fun put_d(a: &mut DU256, i: u64, val: u64) {
        if (i == 0) {
            a.v0 = val;
        } else if (i == 1) {
            a.v1 = val;
        } else if (i == 2) {
            a.v2 = val;
        } else if (i == 3) {
            a.v3 = val;
        } else if (i == 4) {
            a.v4 = val;
        } else if (i == 5) {
            a.v5 = val;
        } else if (i == 6) {
            a.v6 = val;
        } else if (i == 7) {
            a.v7 = val;
        } else {
            abort EWORDS_OVERFLOW
        }
    }

    /// Convert `DU256` to `U256`.
    fun du256_to_u256(a: DU256): (U256, bool) {
        let b = U256 {
            v0: a.v0,
            v1: a.v1,
            v2: a.v2,
            v3: a.v3,
        };

        let overflow = false;
        if (a.v4 != 0 || a.v5 != 0 || a.v6 != 0 || a.v7 != 0) {
            overflow = true;
        };

        (b, overflow)
    }

    // API
    fun from_address(addr: &signer): U256 {
        let encoded = std::bcs::to_bytes(addr);
        from_bytes(&encoded, 0)
    }

    // API
    fun from_bytes(bytes: &vector<u8>, offset: u64): U256 {
        return U256 {
            v0: read_u64(bytes, offset + 24),
            v1: read_u64(bytes, offset + 16),
            v2: read_u64(bytes, offset + 8),
            v3: read_u64(bytes, offset + 0),
        }
    }

    fun from_u64s(v0: u64, v1: u64, v2: u64, v3: u64): U256 {
        return U256 {
            v0,
            v1,
            v2,
            v3,
        }
    }

    // API
    fun from_bool(b: bool): U256 {
        if (b) {
            U256 {
                v0: 1,
                v1: 0,
                v2: 0,
                v3: 0,
            }
        } else {
            U256 {
                v0: 0,
                v1: 0,
                v2: 0,
                v3: 0,
            }
        }
    }

    // API
    fun to_bool(a: U256): bool {
        if (a.v0 == 0 && a.v1 == 0 && a.v2 == 0 && a.v3 == 0) {
            false
        } else {
            true
        }
    }

    fun read_u64(bytes: &vector<u8>, offset: u64): u64 {
        let result = 0u64;
        let i = 0u64;
        while (i < 8) {
            let byte = (*std::vector::borrow(bytes, offset + i) as u64);
            let shift = (((7 - i) * 8) as u8);
            result = result | byte << shift;
            i = i + 1;
        };
        return result
    }

    fun to_bytes(a: &U256): vector<u8> {
        let bytes = std::vector::empty<u8>();
        to_bytes_u64(a.v3, &mut bytes);
        to_bytes_u64(a.v2, &mut bytes);
        to_bytes_u64(a.v1, &mut bytes);
        to_bytes_u64(a.v0, &mut bytes);
        bytes
    }

    fun to_bytes_u64(a: u64, bytes: &mut vector<u8>) {
        std::vector::push_back(bytes, (((a >> 56) & 0xFF) as u8));
        std::vector::push_back(bytes, (((a >> 48) & 0xFF) as u8));
        std::vector::push_back(bytes, (((a >> 40) & 0xFF) as u8));
        std::vector::push_back(bytes, (((a >> 32) & 0xFF) as u8));
        std::vector::push_back(bytes, (((a >> 24) & 0xFF) as u8));
        std::vector::push_back(bytes, (((a >> 16) & 0xFF) as u8));
        std::vector::push_back(bytes, (((a >> 8) & 0xFF) as u8));
        std::vector::push_back(bytes, ((a & 0xFF) as u8));
    }

    fun write(a: &U256, vec: &mut vector<u8>, offset: u64) {
        write_u64(a.v3, vec, offset);
        write_u64(a.v2, vec, offset + 8);
        write_u64(a.v1, vec, offset + 16);
        write_u64(a.v0, vec, offset + 24);
    }

    fun write_u64(a: u64, vec: &mut vector<u8>, offset: u64) {
        *std::vector::borrow_mut(vec, offset + 0) = (((a >> 56) & 0xFF) as u8);
        *std::vector::borrow_mut(vec, offset + 1) = (((a >> 48) & 0xFF) as u8);
        *std::vector::borrow_mut(vec, offset + 2) = (((a >> 40) & 0xFF) as u8);
        *std::vector::borrow_mut(vec, offset + 3) = (((a >> 32) & 0xFF) as u8);
        *std::vector::borrow_mut(vec, offset + 4) = (((a >> 24) & 0xFF) as u8);
        *std::vector::borrow_mut(vec, offset + 5) = (((a >> 16) & 0xFF) as u8);
        *std::vector::borrow_mut(vec, offset + 6) = (((a >> 8) & 0xFF) as u8);
        *std::vector::borrow_mut(vec, offset + 7) = ((a & 0xFF) as u8);
    }

    #[test(self = @0x42)]
    fun test_address_to_u128_1(self: &signer) {
        assert!(from_address(self) == from_u128(0x42), 0);
        let addr = from_address(self);
        let bytes = to_bytes(&addr);
        let val = *std::vector::borrow(&bytes, 7);
        assert!(from_bytes(&bytes, 0) == addr, (val as u64));
        write(&addr, &mut bytes, 0);
        assert!(from_bytes(&bytes, 0) == addr, (val as u64));
    }

    #[test(self = @0x4213421342134213)]
    fun test_address_to_u128_2(self: &signer) {
        assert!(from_address(self) == from_u128(0x4213421342134213), 0);
        let addr = from_address(self);
        let bytes = to_bytes(&addr);
        let val = *std::vector::borrow(&bytes, 7);
        assert!(from_bytes(&bytes, 0) == addr, (val as u64));
        write(&addr, &mut bytes, 0);
        assert!(from_bytes(&bytes, 0) == addr, (val as u64));
    }

    #[test(self = @0xffeeddccbbaa99887766554433221100f0e0d0c0b0a090807060504030201000)]
    fun test_encode_decode(self: &signer) {
        let addr = from_address(self);
        let bytes = to_bytes(&addr);
        let val = *std::vector::borrow(&bytes, 7);
        assert!(from_bytes(&bytes, 0) == addr, (val as u64));
        write(&addr, &mut bytes, 0);
        assert!(from_bytes(&bytes, 0) == addr, 1);
    }

    #[test]
    fun test_from_bool() {
        assert!(from_bool(true) == from_u128(1), 0);
        assert!(from_bool(false) == from_u128(0), 0);
    }

    // Tests.
    #[test]
    fun test_get_d() {
        let a = DU256 {
            v0: 1,
            v1: 2,
            v2: 3,
            v3: 4,
            v4: 5,
            v5: 6,
            v6: 7,
            v7: 8,
        };

        assert!(get_d(&a, 0) == 1, 0);
        assert!(get_d(&a, 1) == 2, 1);
        assert!(get_d(&a, 2) == 3, 2);
        assert!(get_d(&a, 3) == 4, 3);
        assert!(get_d(&a, 4) == 5, 4);
        assert!(get_d(&a, 5) == 6, 5);
        assert!(get_d(&a, 6) == 7, 6);
        assert!(get_d(&a, 7) == 8, 7);
    }

    #[test]
    #[expected_failure(abort_code = 1)]
    fun test_get_d_overflow() {
        let a = DU256 {
            v0: 1,
            v1: 2,
            v2: 3,
            v3: 4,
            v4: 5,
            v5: 6,
            v6: 7,
            v7: 8,
        };

        get_d(&a, 8);
    }

    #[test]
    fun test_exp() {
        let a = from_u128(2);
        let b = from_u128(3);
        let c = exp(a, b);
        assert!(c == from_u128(8), 0);
    }

    #[test]
    fun test_put_d() {
        let a = DU256 {
            v0: 1,
            v1: 2,
            v2: 3,
            v3: 4,
            v4: 5,
            v5: 6,
            v6: 7,
            v7: 8,
        };

        put_d(&mut a, 0, 10);
        put_d(&mut a, 1, 20);
        put_d(&mut a, 2, 30);
        put_d(&mut a, 3, 40);
        put_d(&mut a, 4, 50);
        put_d(&mut a, 5, 60);
        put_d(&mut a, 6, 70);
        put_d(&mut a, 7, 80);

        assert!(get_d(&a, 0) == 10, 0);
        assert!(get_d(&a, 1) == 20, 1);
        assert!(get_d(&a, 2) == 30, 2);
        assert!(get_d(&a, 3) == 40, 3);
        assert!(get_d(&a, 4) == 50, 4);
        assert!(get_d(&a, 5) == 60, 5);
        assert!(get_d(&a, 6) == 70, 6);
        assert!(get_d(&a, 7) == 80, 7);
    }

    #[test]
    #[expected_failure(abort_code = 1)]
    fun test_put_d_overflow() {
        let a = DU256 {
            v0: 1,
            v1: 2,
            v2: 3,
            v3: 4,
            v4: 5,
            v5: 6,
            v6: 7,
            v7: 8,
        };

        put_d(&mut a, 8, 0);
    }

    #[test]
    fun test_du256_to_u256() {
        let a = DU256 {
            v0: 255,
            v1: 100,
            v2: 50,
            v3: 300,
            v4: 0,
            v5: 0,
            v6: 0,
            v7: 0,
        };

        let (m, overflow) = du256_to_u256(a);
        assert!(!overflow, 0);
        assert!(m.v0 == a.v0, 1);
        assert!(m.v1 == a.v1, 2);
        assert!(m.v2 == a.v2, 3);
        assert!(m.v3 == a.v3, 4);

        a.v4 = 100;
        a.v5 = 5;

        let (m, overflow) = du256_to_u256(a);
        assert!(overflow, 5);
        assert!(m.v0 == a.v0, 6);
        assert!(m.v1 == a.v1, 7);
        assert!(m.v2 == a.v2, 8);
        assert!(m.v3 == a.v3, 9);
    }

    #[test]
    fun test_get() {
        let a = U256 {
            v0: 1,
            v1: 2,
            v2: 3,
            v3: 4,
        };

        assert!(get(&a, 0) == 1, 0);
        assert!(get(&a, 1) == 2, 1);
        assert!(get(&a, 2) == 3, 2);
        assert!(get(&a, 3) == 4, 3);
    }

    #[test]
    #[expected_failure(abort_code = 1)]
    fun test_get_aborts() {
        let _ = get(&zero(), 4);
    }

    #[test]
    fun test_put() {
        let a = zero();
        put(&mut a, 0, 255);
        assert!(get(&a, 0) == 255, 0);

        put(&mut a, 1, (U64_MAX as u64));
        assert!(get(&a, 1) == (U64_MAX as u64), 1);

        put(&mut a, 2, 100);
        assert!(get(&a, 2) == 100, 2);

        put(&mut a, 3, 3);
        assert!(get(&a, 3) == 3, 3);

        put(&mut a, 2, 0);
        assert!(get(&a, 2) == 0, 4);
    }

    #[test]
    fun test_one() {
        let a = one();
        assert!(eq(a, from_u128(1)), 0);
    }

    #[test]
    #[expected_failure(abort_code = 1)]
    fun test_put_overflow() {
        let a = zero();
        put(&mut a, 6, 255);
    }

    #[test]
    fun test_from_u128() {
        let i = 0;
        while (i < 1024) {
            let big = from_u128(i);
            assert!(as_u128(big) == i, 0);
            i = i + 1;
        };
    }

    #[test]
    fun test_add() {
        let a = from_u128(1000);
        let b = from_u128(500);

        let s = as_u128(overflowing_add(a, b));
        assert!(s == 1500, 0);

        a = from_u128(U64_MAX);
        b = from_u128(U64_MAX);

        s = as_u128(overflowing_add(a, b));
        assert!(s == (U64_MAX + U64_MAX), 1);
    }

    #[test]
    fun test_add_overflow() {
        let max = (U64_MAX as u64);

        let a = U256 {
            v0: max,
            v1: max,
            v2: max,
            v3: max
        };

        let _ = overflowing_add(a, from_u128(1));
    }

    #[test]
    fun test_sub() {
        let a = from_u128(1000);
        let b = from_u128(500);

        let s = as_u128(overflowing_sub(a, b));
        assert!(s == 500, 0);
    }

    #[test]
    fun test_overflowing_add() {
        let (n, z) = overflowing_add_u64(10, 10);
        assert!(n == 20, 0);
        assert!(!z, 1);

        (n, z) = overflowing_add_u64((U64_MAX as u64), 1);
        assert!(n == 0, 2);
        assert!(z, 3);

        (n, z) = overflowing_add_u64((U64_MAX as u64), 10);
        assert!(n == 9, 4);
        assert!(z, 5);

        (n, z) = overflowing_add_u64(5, 8);
        assert!(n == 13, 6);
        assert!(!z, 7);
    }

    #[test]
    fun test_overflowing_sub() {
        let (n, z) = overflowing_sub_u64(10, 5);
        assert!(n == 5, 0);
        assert!(!z, 1);

        (n, z) = overflowing_sub_u64(0, 1);
        assert!(n == (U64_MAX as u64), 2);
        assert!(z, 3);

        (n, z) = overflowing_sub_u64(10, 10);
        assert!(n == 0, 4);
        assert!(!z, 5);
    }

    #[test]
    fun test_split_u128() {
        let (a1, a2) = split_u128(100);
        assert!(a1 == 0, 0);
        assert!(a2 == 100, 1);

        (a1, a2) = split_u128(U64_MAX + 1);
        assert!(a1 == 1, 2);
        assert!(a2 == 0, 3);
    }

    #[test]
    fun test_mul() {
        let a = from_u128(285);
        let b = from_u128(375);

        let c = as_u128(overflowing_mul(a, b));
        assert!(c == 106875, 0);

        a = from_u128(0);
        b = from_u128(1);

        c = as_u128(overflowing_mul(a, b));

        assert!(c == 0, 1);

        a = from_u128(U64_MAX);
        b = from_u128(2);

        c = as_u128(overflowing_mul(a, b));

        assert!(c == 36893488147419103230, 2);

        a = from_u128(U128_MAX);
        b = from_u128(U128_MAX);

        let z = overflowing_mul(a, b);
        assert!(bits(&z) == 256, 3);
    }

    #[test]
    fun test_mul_overflow() {
        let max = (U64_MAX as u64);

        let a = U256 {
            v0: max,
            v1: max,
            v2: max,
            v3: max,
        };

        let _ = overflowing_mul(a, from_u128(2));
    }

    #[test]
    fun test_zero() {
        let a = as_u128(zero());
        assert!(a == 0, 0);

        let a = zero();
        assert!(a.v0 == 0, 1);
        assert!(a.v1 == 0, 2);
        assert!(a.v2 == 0, 3);
        assert!(a.v3 == 0, 4);
    }

    #[test]
    fun test_compare() {
        let a = from_u128(1000);
        let b = from_u128(50);

        let cmp = compare(&a, &b);
        assert!(cmp == 2, 0);

        a = from_u128(100);
        b = from_u128(100);
        cmp = compare(&a, &b);

        assert!(cmp == 0, 1);

        a = from_u128(50);
        b = from_u128(75);

        cmp = compare(&a, &b);
        assert!(cmp == 1, 2);
    }

    #[test]
    fun test_leading_zeros_u64() {
        let a = leading_zeros_u64(0);
        assert!(a == 64, 0);

        let a = leading_zeros_u64(1);
        assert!(a == 63, 1);

        // TODO: more tests.
    }

    #[test]
    fun test_bits() {
        let a = bits(&from_u128(0));
        assert!(a == 0, 0);

        a = bits(&from_u128(255));
        assert!(a == 8, 1);

        a = bits(&from_u128(256));
        assert!(a == 9, 2);

        a = bits(&from_u128(300));
        assert!(a == 9, 3);

        a = bits(&from_u128(60000));
        assert!(a == 16, 4);

        a = bits(&from_u128(70000));
        assert!(a == 17, 5);

        let b = from_u128(70000);
        let sh = shl_u8(b, 100);
        assert!(bits(&sh) == 117, 6);

        let sh = shl_u8(sh, 100);
        assert!(bits(&sh) == 217, 7);

        let sh = shl_u8(sh, 100);
        assert!(bits(&sh) == 0, 8);
    }

    #[test]
    fun test_shift_left() {
        let a = from_u128(100);
        let b = shl_u8(a, 2);

        assert!(as_u128(b) == 400, 0);
    }

    #[test]
    fun test_shift_right() {
        let a = from_u128(100);
        let b = shr_u8(a, 2);

        assert!(as_u128(b) == 25, 0);
    }

    #[test]
    fun test_div() {
        let a = from_u128(100);
        let b = from_u128(5);
        let d = div(a, b);

        assert!(as_u128(d) == 20, 0);

        let a = from_u128(U64_MAX);
        let b = from_u128(U128_MAX);
        let d = div(a, b);
        assert!(as_u128(d) == 0, 1);

        let a = from_u128(U64_MAX);
        let b = from_u128(U128_MAX);
        let d = div(a, b);
        assert!(as_u128(d) == 0, 2);

        let a = from_u128(U128_MAX);
        let b = from_u128(U64_MAX);
        let d = div(a, b);
        assert!(as_u128(d) == 18446744073709551617, 2);
    }

    #[test]
    fun test_div_by_zero() {
        let a = from_u128(1);
        let z = div(a, from_u128(0));
        assert!(as_u128(z) == 0, 0);
    }

    #[test]
    fun test_mod() {
        let a = from_u128(100);
        let b = from_u128(5);
        let d = mod(a, b);
        assert!(as_u128(d) == 0, 0);

        let a = from_u128(100);
        let b = from_u128(0);
        let d = mod(a, b);
        assert!(as_u128(d) == 0, 0);

        let a = from_u128(100);
        let b = from_u128(51);
        let d = mod(a, b);
        assert!(as_u128(d) == 49, 0);
    }

    #[test]
    fun test_or() {
        let a = from_u128(0);
        let b = from_u128(1);
        let c = bitor(a, b);
        assert!(as_u128(c) == 1, 0);

        let a = from_u128(0x0f0f0f0f0f0f0f0fu128);
        let b = from_u128(0xf0f0f0f0f0f0f0f0u128);
        let c = bitor(a, b);
        assert!(as_u128(c) == 0xffffffffffffffffu128, 1);
    }

    #[test]
    fun test_and() {
        let a = from_u128(0);
        let b = from_u128(1);
        let c = bitand(a, b);
        assert!(as_u128(c) == 0, 0);

        let a = from_u128(0x0f0f0f0f0f0f0f0fu128);
        let b = from_u128(0xf0f0f0f0f0f0f0f0u128);
        let c = bitand(a, b);
        assert!(as_u128(c) == 0, 1);

        let a = from_u128(0x0f0f0f0f0f0f0f0fu128);
        let b = from_u128(0x0f0f0f0f0f0f0f0fu128);
        let c = bitand(a, b);
        assert!(as_u128(c) == 0x0f0f0f0f0f0f0f0fu128, 1);
    }

    #[test]
    fun test_xor() {
        let a = from_u128(0);
        let b = from_u128(1);
        let c = bitxor(a, b);
        assert!(as_u128(c) == 1, 0);

        let a = from_u128(0x0f0f0f0f0f0f0f0fu128);
        let b = from_u128(0xf0f0f0f0f0f0f0f0u128);
        let c = bitxor(a, b);
        assert!(as_u128(c) == 0xffffffffffffffffu128, 1);
    }
}
