module self::math {

    /// Max `u64` value.
    const U64_MAX: u128 = 18446744073709551615;

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

    // Tests.

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
    fun test_leading_zeros_u64() {
        let a = leading_zeros_u64(0);
        assert!(a == 64, 0);

        let a = leading_zeros_u64(1);
        assert!(a == 63, 1);

        // TODO: more tests.
    }
}