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

    /// Compares two `U256` numbers.
    fun compare(a: & U256, b: & U256): u8 {
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
        b = shl(b, (shift as u8));

        loop {
            let cmp = compare(&a, &b);
            if (cmp == GREATER_THAN || cmp == EQUAL) {
                let index = shift / 64;
                let m = get(&ret, index);
                let c = m | 1 << ((shift % 64) as u8);
                put(&mut ret, index, c);

                a = overflowing_sub(a, b);
            };

            b = shr(b, 1);
            if (shift == 0) {
                break
            };

            shift = shift - 1;
        };

        ret
    }

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
        b = shl(b, (shift as u8));

        loop {
            let cmp = compare(&a, &b);
            if (cmp == GREATER_THAN || cmp == EQUAL) {
                a = overflowing_sub(a, b);
            };

            b = shr(b, 1);
            if (shift == 0) {
                break
            };

            shift = shift - 1;
        };

        a
    }

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

    /// Shift right `a`  by `shift`.
    fun shr(a: U256, shift: u8): U256 {
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

    /// Shift left `a` by `shift`.
    fun shl(a: U256, shift: u8): U256 {
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
    fun bits(a: & U256): u64 {
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
    fun get(a: & U256, i: u64): u64 {
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
        let sh = shl(b, 100);
        assert!(bits(&sh) == 117, 6);

        let sh = shl(sh, 100);
        assert!(bits(&sh) == 217, 7);

        let sh = shl(sh, 100);
        assert!(bits(&sh) == 0, 8);
    }

    #[test]
    fun test_shift_left() {
        let a = from_u128(100);
        let b = shl(a, 2);

        assert!(as_u128(b) == 400, 0);

        // TODO: more shift left tests.
    }

    #[test]
    fun test_shift_right() {
        let a = from_u128(100);
        let b = shr(a, 2);

        assert!(as_u128(b) == 25, 0);

        // TODO: more shift right tests.
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
}
