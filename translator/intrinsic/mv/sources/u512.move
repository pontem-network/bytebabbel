module self::u512 {
    // Errors.
    /// When trying to get or put word into U256 but it's out of index.
    const EWORDS_OVERFLOW: u64 = 1;

    /// When trying to divide or get module by zero
    const EDIV: u64 = 2;

    /// Total words in `U512` (64 * 8 = 512).
    const DWORDS: u64 = 8;

    /// Total words in `U256` (64 * 4 = 256).
    const WORDS: u64 = 4;

    /// When both `U256` equal.
    const EQUAL: u8 = 0;

    /// When `a` is less than `b`.
    const LESS_THAN: u8 = 1;

    /// When `a` is greater than `b`.
    const GREATER_THAN: u8 = 2;


    /// Double `U256` used for multiple (to store overflow).
    struct U512 has copy, drop, store {
        v0: u64,
        v1: u64,
        v2: u64,
        v3: u64,
        v4: u64,
        v5: u64,
        v6: u64,
        v7: u64,
    }

    public fun new_u512(v0: u64, v1: u64, v2: u64, v3: u64, v4: u64, v5: u64, v6: u64, v7: u64): U512 {
        U512 {
            v0,
            v1,
            v2,
            v3,
            v4,
            v5,
            v6,
            v7,
        }
    }

    /// Get word from `U512` by index.
    public fun get_d(a: &U512, i: u64): u64 {
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

    /// Put new word into `U512` by index `i`.
    public fun put_d(a: &mut U512, i: u64, val: u64) {
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

    public fun zero_d(): U512 {
        U512 {
            v0:0,
            v1:0,
            v2:0,
            v3:0,
            v4:0,
            v5:0,
            v6:0,
            v7:0,
        }
    }

    /// Compares two `U512` numbers.
    public fun compare_d(a: &U512, b: &U512): u8 {
        let i = DWORDS;
        while (i > 0) {
            i = i - 1;
            let a1 = get_d(a, i);
            let b1 = get_d(b, i);

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

    use self::utiles::overflowing_add_u64;

    /// Add two `U512`, returns result.
    public fun overflowing_add_d(a: U512, b: U512): U512 {
        let ret = zero_d();
        let carry = 0u64;

        let i = 0;
        while (i < DWORDS) {
            let a1 = get_d(&a, i);
            let b1 = get_d(&b, i);

            if (carry != 0) {
                let (res1, is_overflow1) = overflowing_add_u64(a1, b1);
                let (res2, is_overflow2) = overflowing_add_u64(res1, carry);
                put_d(&mut ret, i, res2);

                carry = 0;
                if (is_overflow1) {
                    carry = carry + 1;
                };

                if (is_overflow2) {
                    carry = carry + 1;
                }
            } else {
                let (res, is_overflow) = overflowing_add_u64(a1, b1);
                put_d(&mut ret, i, res);

                carry = 0;
                if (is_overflow) {
                    carry = 1;
                };
            };

            i = i + 1;
        };
        ret
    }

    use self::utiles::overflowing_sub_u64;

    /// Subtracts two `U512`, returns result.
    public fun overflowing_sub_d(a: U512, b: U512): U512 {
        let ret = zero_d();

        let carry = 0u64;

        let i = 0;
        while (i < DWORDS) {
            let a1 = get_d(&a, i);
            let b1 = get_d(&b, i);

            if (carry != 0) {
                let (res1, is_overflow1) = overflowing_sub_u64(a1, b1);
                let (res2, is_overflow2) = overflowing_sub_u64(res1, carry);
                put_d(&mut ret, i, res2);

                carry = 0;
                if (is_overflow1) {
                    carry = carry + 1;
                };

                if (is_overflow2) {
                    carry = carry + 1;
                }
            } else {
                let (res, is_overflow) = overflowing_sub_u64(a1, b1);
                put_d(&mut ret, i, res);

                carry = 0;
                if (is_overflow) {
                    carry = 1;
                };
            };

            i = i + 1;
        };
        ret
    }

    /// Mod `a` by `b` in U512.
    public fun mod_d(a: U512, b: U512): U512 {
        let a_bits = bits_d(&a);
        let b_bits = bits_d(&b);

        if (b_bits == 0) {
            return zero_d()
        };

        if (a_bits < b_bits) {
            return a
        };

        let shift = a_bits - b_bits;
        b = shl_u8_d(b, (shift as u64));

        loop {
            let cmp = compare_d(&a, &b);
            if (cmp == GREATER_THAN || cmp == EQUAL) {
                a = overflowing_sub_d(a, b);
            };

            b = shr_u8_d(b, 1);
            if (shift == 0) {
                break
            };

            shift = shift - 1;
        };

        a
    }

    use self::utiles::leading_zeros_u64;

    /// Get bits used to store `a` in U512.
    public fun bits_d(a: &U512): u64 {
        let i = 1;
        while (i < DWORDS) {
            let a1 = get_d(a, DWORDS - i);
            if (a1 > 0) {
                return ((0x40 * (DWORDS - i + 1)) - (leading_zeros_u64(a1) as u64))
            };

            i = i + 1;
        };

        let a1 = get_d(a, 0);
        0x40 - (leading_zeros_u64(a1) as u64)
    }

    /// Shift left `a` by `shift` in U512.
    public fun shl_u8_d(a: U512, shift: u64): U512 {
        let ret = zero_d();

        let word_shift = (shift as u64) / 64;
        let bit_shift = (shift as u64) % 64;

        let i = word_shift;
        while (i < DWORDS) {
            let m = get_d(&a, i - word_shift) << (bit_shift as u8);
            put_d(&mut ret, i, m);
            i = i + 1;
        };

        if (bit_shift > 0) {
            let j = word_shift + 1;

            while (j < DWORDS) {
                let m = get_d(&ret, j) + (get_d(&a, j - 1 - word_shift) >> (64 - (bit_shift as u8)));
                put_d(&mut ret, j, m);
                j = j + 1;
            };
        };

        ret
    }

    /// Shift right `a`  by `shift` in U512.
    public fun shr_u8_d(a: U512, shift: u8): U512 {
        let ret = zero_d();

        let word_shift = (shift as u64) / 64;
        let bit_shift = (shift as u64) % 64;

        let i = word_shift;
        while (i < DWORDS) {
            let m = get_d(&a, i) >> (bit_shift as u8);
            put_d(&mut ret, i - word_shift, m);
            i = i + 1;
        };

        if (bit_shift > 0) {
            let j = word_shift + 1;
            while (j < DWORDS) {
                let m = get_d(&ret, j - word_shift - 1) + (get_d(&a, j) << (64 - (bit_shift as u8)));
                put_d(&mut ret, j - word_shift - 1, m);
                j = j + 1;
            };
        };

        ret
    }
}