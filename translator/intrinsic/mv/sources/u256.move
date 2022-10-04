module self::u256 {
    // U256.
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

    /// When `a` is greater than `b`.
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

    public fun new_u256(v0: u64, v1: u64, v2: u64, v3: u64): U256 {
        U256 {
            v0,
            v1,
            v2,
            v3,
        }
    }

    public fun from_string(string: &vector<u8>): U256 {
        let sign = true;
        let shift = 0;
        let lenght = std::vector::length(string);
        let res = zero();
        let ten = from_u128(10);

        // char '-' = 45
        if (*std::vector::borrow(string, shift) == 45) {
            sign = false;
            shift = shift + 1;
        };

        // char '0' = 48
        while (shift < lenght) {
            let digit = *std::vector::borrow(string, shift) - 48;

            res = overflowing_add(
                overflowing_mul(res, ten),
                from_u128((digit as u128))
            );
            shift = shift + 1;
        };

        if (sign) {
            res
        } else {
            get_negative(res)
        }
    }

    /// Returns `U256` equals to zero.
    public fun zero(): U256 {
        U256 {
            v0: 0,
            v1: 0,
            v2: 0,
            v3: 0,
        }
    }

    /// Returns a `U256` from `u128` value.
    public fun from_u128(val: u128): U256 {
        let (a2, a1) = split_u128(val);
        U256 {
            v0: a1,
            v1: a2,
            v2: 0,
            v3: 0,
        }
    }

    // API
    // TODO
    public fun from_signer(addr: &signer): U256 {
        let encoded = std::bcs::to_bytes(addr);
        // todo replace with riding last 20 bytes
        let address_mask = U256 {
            v0: 0xFFFFFFFFFFFFFFFF,
            v1: 0xFFFFFFFFFFFFFFFF,
            v2: 0x00000000FFFFFFFF,
            v3: 0x0000000000000000,
        };
        bitand(from_bytes(&encoded, zero()), address_mask)
    }

    // API
    // TODO
    fun from_address(addr: address): U256 {
        let encoded = std::bcs::to_bytes(&addr);
        // todo replace with riding last 20 bytes
        let address_mask = U256 {
            v0: 0xFFFFFFFFFFFFFFFF,
            v1: 0xFFFFFFFFFFFFFFFF,
            v2: 0x00000000FFFFFFFF,
            v3: 0x0000000000000000,
        };
        bitand(from_bytes(&encoded, zero()), address_mask)
    }

    // API
    public fun from_bytes(bytes: &vector<u8>, offset: U256): U256 {
        let offset = offset.v0;
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
    public fun from_bool(b: bool): U256 {
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

    public fun to_bytes(a: &U256): vector<u8> {
        let bytes = std::vector::empty<u8>();
        to_bytes_u64(a.v3, &mut bytes);
        to_bytes_u64(a.v2, &mut bytes);
        to_bytes_u64(a.v1, &mut bytes);
        to_bytes_u64(a.v0, &mut bytes);
        bytes
    }

    /// Convert `U256` to `u128` value if possible (otherwise it aborts).
    public fun as_u128(a: U256): u128 {
        assert!(a.v2 == 0 && a.v3 == 0, EWORDS_OVERFLOW);
        ((a.v1 as u128) << 64) + (a.v0 as u128)
    }

    /// Convert `U256` to `u64`
    public fun as_u64(a: U256): u64 {
        a.v0
    }

    /// API
    fun to_address(a: U256): address {
        let encoded = to_bytes(&a);
        return aptos_framework::util::address_from_bytes(encoded)
    }

    /// Get word from `a` by index `i`.
    public fun get(a: &U256, i: u64): u64 {
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

    /// Put new word `val` into `U256` by index `i`.
    public fun put(a: &mut U256, i: u64, val: u64) {
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

    // API
    /// Divide `a` by `b`.
    public fun div(a: U256, b: U256): U256 {
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
    /// Binary xor `a` by `b`.
    public fun bitxor(a: U256, b: U256): U256 {
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

    // API
    /// Binary and `a` by `b`.
    public fun bitand(a: U256, b: U256): U256 {
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
    /// Binary or `a` by `b`.
    public fun bitor(a: U256, b: U256): U256 {
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
    /// Mod `a` by `b`.
    public fun mod(a: U256, b: U256): U256 {
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

    /// Shift left `a` by `shift`.
    public fun shl_u8(a: U256, shift: u8): U256 {
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

    // Private functions.
    /// Get bits used to store `a`.
    public fun bits(a: &U256): u64 {
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

    // API
    /// Signed lt.
    public fun slt(a: U256, b: U256): bool {
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
    public fun eq(a: U256, b: U256): bool {
        compare(&a, &b) == EQUAL
    }

    // API
    fun ne(a: U256, b: U256): bool {
        compare(&a, &b) != EQUAL
    }

    // API
    public fun bitnot(a: U256): U256 {
        let ret = zero();
        let i = 0;
        while (i < WORDS) {
            put(&mut ret, i, get(&a, i) ^ 0xFFFFFFFFFFFFFFFF);
            i = i + 1;
        };
        ret
    }

    // API
    public fun byte(i: U256, x: U256): U256 {
        let shift = 248 - as_u128(i) * 8;
        bitand(shr_u8(x, (shift as u8)), from_u128(0xFF))
    }

    /// Compares two `U256` numbers.
    public fun compare(a: &U256, b: &U256): u8 {
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

    // API
    /// Adds two `U256` and returns sum.
    public fun overflowing_add(a: U256, b: U256): U256 {
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

    use self::du256::{zero_d, get_d, put_d, DU256};

    // API
    /// Multiples two `U256`.
    public fun overflowing_mul(a: U256, b: U256): U256 {
        let ret = zero_d();

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
    public fun overflowing_sub(a: U256, b: U256): U256 {
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
    /// Divide `a` by `b` with sign.
    public fun sdiv(a: U256, b: U256): U256 {
        let a_neg = is_negative(&a);
        let b_neg = is_negative(&b);

        let a = if (a_neg) { get_negative(a) } else { a };
        let b = if (b_neg) { get_negative(b) } else { b };
    
        let ret = div(a, b);

        if (a_neg != b_neg) {
            get_negative(ret)
        } else {
            ret
        }
    }

    // API
    /// Signed gt.
    public fun sgt(a: U256, b: U256): bool {
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
    public fun smod(a: U256, b: U256): U256 {
        let a_neg = is_negative(&a);
        let b_neg = is_negative(&b);

        let a = if (a_neg) { get_negative(a) } else { a };
        let b = if (b_neg) { get_negative(b) } else { b };

        let ret = mod(a, b);

        if (compare(&ret, &zero()) == EQUAL) {
            zero()
        } else if (a_neg) {
            get_negative(ret)
        } else {
            ret
        }
    }

    // API
    /// Exponentiation.
    /// todo use DU256 for intermediate calculations
    public fun exp(a: U256, b: U256): U256 {
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

    public fun one(): U256 {
        U256 {
            v0: 1,
            v1: 0,
            v2: 0,
            v3: 0,
        }
    }

    public fun is_negative(a: &U256): bool {
        let msb = get(a, WORDS - 1);
        msb & 0x8000000000000000 != 0
    }

    // change sign
    public fun get_negative(a: U256): U256 {
        if (is_negative(&a)) {
            bitnot(overflowing_sub(a, from_u128(1)))
        } else {
            overflowing_add(bitnot(a), from_u128(1))
        }
    }

    /// Shift right `a`  by `shift`.
    public fun shr_u8(a: U256, shift: u8): U256 {
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

    public fun write(a: &U256, vec: &mut vector<u8>, offset: u64) {
        write_u64(a.v3, vec, offset);
        write_u64(a.v2, vec, offset + 8);
        write_u64(a.v1, vec, offset + 16);
        write_u64(a.v0, vec, offset + 24);
    }

    public fun read_u64(bytes: &vector<u8>, offset: u64): u64 {
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

    public fun to_bytes_u64(a: u64, bytes: &mut vector<u8>) {
        std::vector::push_back(bytes, (((a >> 56) & 0xFF) as u8));
        std::vector::push_back(bytes, (((a >> 48) & 0xFF) as u8));
        std::vector::push_back(bytes, (((a >> 40) & 0xFF) as u8));
        std::vector::push_back(bytes, (((a >> 32) & 0xFF) as u8));
        std::vector::push_back(bytes, (((a >> 24) & 0xFF) as u8));
        std::vector::push_back(bytes, (((a >> 16) & 0xFF) as u8));
        std::vector::push_back(bytes, (((a >> 8) & 0xFF) as u8));
        std::vector::push_back(bytes, ((a & 0xFF) as u8));
    }

    public fun write_u64(a: u64, vec: &mut vector<u8>, offset: u64) {
        *std::vector::borrow_mut(vec, offset + 0) = (((a >> 56) & 0xFF) as u8);
        *std::vector::borrow_mut(vec, offset + 1) = (((a >> 48) & 0xFF) as u8);
        *std::vector::borrow_mut(vec, offset + 2) = (((a >> 40) & 0xFF) as u8);
        *std::vector::borrow_mut(vec, offset + 3) = (((a >> 32) & 0xFF) as u8);
        *std::vector::borrow_mut(vec, offset + 4) = (((a >> 24) & 0xFF) as u8);
        *std::vector::borrow_mut(vec, offset + 5) = (((a >> 16) & 0xFF) as u8);
        *std::vector::borrow_mut(vec, offset + 6) = (((a >> 8) & 0xFF) as u8);
        *std::vector::borrow_mut(vec, offset + 7) = ((a & 0xFF) as u8);
    }

    /// Get leading zeros of a binary representation of `a`.
    public fun leading_zeros_u64(a: u64): u8 {
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
    public fun overflowing_add_u64(a: u64, b: u64): (u64, bool) {
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
    public fun overflowing_sub_u64(a: u64, b: u64): (u64, bool) {
        if (a < b) {
            let r = b - a;
            ((U64_MAX as u64) - r + 1, true)
        } else {
            (a - b, false)
        }
    }

    /// Extracts two `u64` from `a` `u128`.
    public fun split_u128(a: u128): (u64, u64) {
        let a1 = ((a >> 64) as u64);
        let a2 = ((a & 0xFFFFFFFFFFFFFFFF) as u64);

        (a1, a2)
    }


    /// Convert `DU256` to `U256`.
    public fun du256_to_u256(a: DU256): (U256, bool) {
        let b = new_u256(get_d(&a, 0), get_d(&a, 1), get_d(&a, 2), get_d(&a, 3));

        let overflow = false;
        if (get_d(&a, 4) != 0 || get_d(&a, 5) != 0 || get_d(&a, 6) != 0 || get_d(&a, 7) != 0) {
            overflow = true;
        };

        (b, overflow)
    }
}