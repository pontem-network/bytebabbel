module self::u256_tests {
    #[test_only]
    use std::vector;

    /// Max `u64` value.
    const U64_MAX: u128 = 18446744073709551615;

    /// Max `u128` value.
    const U128_MAX: u128 = 340282366920938463463374607431768211455;

    #[test_only]
    use self::u256::read_u64;

    #[test]
    fun test_read_u64() {
        // taking vector with len < 8 leads to panic
        let vec: vector<u8> = vector::empty();
        let i = 0u64;

        while (i < 10) {
            vector::push_back(&mut vec, (i as u8));
            i = i + 1;
        };

        // 00 01 02 03 04 05 06 07 -> 283686952306183
        assert!(read_u64(&vec, 0) == 283686952306183u64, 0);
        assert!(read_u64(&vec, 1) == 283686952306183u64 << 8 | 8, 1);
        assert!(read_u64(&vec, 2) == (283686952306183u64 << 8 | 8) << 8 | 9, 2);
    }

    #[test_only]
    use self::u256::to_bytes_u64;

    #[test]
    fun test_to_bytes_u64() {
        let vec: vector<u8> = vector::empty();

        to_bytes_u64(4096, &mut vec);

        assert!(*vector::borrow(&vec, 6) == 16u8, 0);
        assert!(*vector::borrow(&vec, 7) == 0, 1);
    }

    #[test_only]
    use self::u256::write_u64;

    #[test]
    fun test_write_u64() {
        let vec: vector<u8> = vector[0, 0, 0, 0, 0, 0, 0, 0, 0];

        write_u64(4096, &mut vec, 1);

        assert!(*vector::borrow(&vec, 7) == 16u8, 0);
        assert!(*vector::borrow(&vec, 8) == 0, 1);
    }

    #[test_only]
    use self::utiles::overflowing_add_u64;

    #[test]
    fun test_overflowing_add_u64() {
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

    #[test_only]
    use self::utiles::overflowing_sub_u64;

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

    #[test_only]
    use self::utiles::split_u128;

    #[test]
    fun test_split_u128() {
        let (a1, a2) = split_u128(100);
        assert!(a1 == 0, 0);
        assert!(a2 == 100, 1);

        (a1, a2) = split_u128(U64_MAX + 1);
        assert!(a1 == 1, 2);
        assert!(a2 == 0, 3);
    }

    #[test_only]
    use self::utiles::leading_zeros_u64;

    #[test]
    fun test_leading_zeros_u64() {
        let a = leading_zeros_u64(0);
        assert!(a == 64, 0);

        let a = leading_zeros_u64(1);
        assert!(a == 63, 1);

        let a = leading_zeros_u64(64);
        assert!(a == 57, 2);
        // TODO: more tests.
    }

    #[test_only]
    use self::u256::{from_bool, from_u128};

    #[test]
    fun test_from_bool() {
        assert!(from_bool(true) == from_u128(1), 0);
        assert!(from_bool(false) == from_u128(0), 0);
    }

    #[test_only]
    use self::u256::{as_u64, as_u128};

    #[test]
    fun test_as_u64() {
        let a = from_u128(0);
        let b = as_u64(a);
        assert!(b == 0, 0);

        let a = from_u128(1);
        let b = as_u64(a);
        assert!(b == 1, 1);

        let a = from_u128(0xffffffffffffffff);
        let b = as_u64(a);
        assert!(b == 0xffffffffffffffff, 2);

        let a = from_u128(0x10000000000000000);
        let b = as_u64(a);
        assert!(b == 0, 3);
    }

    #[test_only]
    use self::u256::{zero, get};

    #[test]
    fun test_zero() {
        let a = as_u128(zero());
        assert!(a == 0, 0);

        let a = zero();
        assert!(get(&a, 0) == 0, 1);
        assert!(get(&a, 1) == 0, 2);
        assert!(get(&a, 2) == 0, 3);
        assert!(get(&a, 3) == 0, 4);
    }

    #[test_only]
    use self::u256::div;

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
        assert!(as_u128(d) == 18446744073709551617, 3);
    }

    #[test_only]
    use self::u256::get_negative;

    #[test]
    fun test_negative() {
        let a = from_string(&b"30");
        let b = get_negative(a);
        let c = as_u128(overflowing_add(a, b));

        assert!(c == 0, 0);

        let a = from_u128(0);
        let b = get_negative(a);
        let c = as_u128(overflowing_add(a, b));

        assert!(c == 0, 1);

        let b = from_string(&b"-100");
        let c = overflowing_add(b, b);
        let d = as_u128(get_negative(c));

        assert!(d == 200, 2);
    }

    #[test_only]
    use self::u256::slt;

    #[test]
    fun test_slt() {
        let a = from_u128(0);
        let b = from_u128(1);
        let c = slt(a, b);
        assert!(c == true, 0);

        let a = from_u128(1);
        let b = from_u128(0);
        let c = slt(a, b);
        assert!(c == false, 1);

        let a = from_u128(128);
        let b = from_u128(128);
        let c = slt(a, b);
        assert!(c == false, 2);

        let a = from_string(&b"-1");
        let b = from_string(&b"-2");
        let c = slt(a, b);
        assert!(c == false, 3);

        let a = from_string(&b"-2");
        let b = from_string(&b"1");
        let c = slt(a, b);
        assert!(c == true, 4);

        let a = from_string(&b"-2");
        let b = from_string(&b"-1");
        let c = slt(a, b);
        assert!(c == true, 5);
    }

    #[test_only]
    use self::u256::sgt;

    #[test]
    fun test_sgt() {
        let a = from_u128(0);
        let b = from_u128(1);
        let c = sgt(a, b);
        assert!(c == false, 0);

        let a = from_u128(1);
        let b = from_u128(0);
        let c = sgt(a, b);
        assert!(c == true, 1);

        let a = from_u128(128);
        let b = from_u128(128);
        let c = sgt(a, b);
        assert!(c == false, 2);

        let a = from_string(&b"-1");
        let b = from_string(&b"-2");
        let c = sgt(a, b);
        assert!(c == true, 3);

        let a = from_string(&b"-2");
        let b = from_string(&b"1");
        let c = sgt(a, b);
        assert!(c == false, 4);
    }

    #[test_only]
    use self::u256::{sdiv};

    #[test]
    fun test_sdiv() {
        let a = from_string(&b"-100");
        let b = get_negative(from_u128(5));
        let d = sdiv(a, b);
        assert!(as_u128(d) == 20, 0);

        let a = from_string(&b"-100");
        let b = from_u128(5);
        let d = get_negative(sdiv(a, b));
        assert!(as_u128(d) == 20, 1);

        let a = from_u128(U64_MAX);
        let b = get_negative(from_u128(U128_MAX));
        let d = get_negative(sdiv(a, b));
        assert!(as_u128(d) == 0, 2);

        let a = from_u128(U128_MAX);
        let b = from_u128(U64_MAX);
        let d = sdiv(a, b);
        assert!(as_u128(d) == 18446744073709551617, 3);
    }

    #[test_only]
    use self::u256::smod;

    #[test]
    fun test_smod() {
        let a = from_u128(100);
        let b = from_u128(5);
        let d = smod(a, b);
        assert!(as_u128(d) == 0, 0);

        let a = from_string(&b"-100");
        let b = from_u128(5);
        let d = smod(a, b);
        assert!(as_u128(d) == 0, 1);

        let a = from_u128(100);
        let b = from_string(&b"-5");
        let d = smod(a, b);
        assert!(as_u128(d) == 0, 2);

        let a = from_string(&b"-5");
        let b = from_u128(2);
        let d = get_negative(smod(a, b));
        assert!(as_u128(d) == 1, 3);

        let a = get_negative(from_u128(5));
        let b = get_negative(from_u128(2));
        let d = get_negative(smod(a, b));
        assert!(as_u128(d) == 1, 4);
    }

    #[test_only]
    use self::u256::bitor;

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

    #[test_only]
    use self::u256::bitand;

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

    #[test_only]
    use self::u256::bitxor;

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

    #[test_only]
    use self::u256::shr_u8;

    #[test]
    fun test_shift_right() {
        let a = from_u128(100);
        let b = shr_u8(a, 2);

        assert!(as_u128(b) == 25, 0);
    }

    #[test]
    fun test_div_by_zero() {
        let a = from_u128(1);
        let z = div(a, from_u128(0));

        assert!(as_u128(z) == 0, 0);
    }

    #[test_only]
    use self::u256::mod;

    #[test]
    fun test_mod() {
        let a = from_u128(100);
        let b = from_u128(5);
        let d = mod(a, b);
        assert!(as_u128(d) == 0, 0);

        let a = from_u128(100);
        let b = from_u128(0);
        let d = mod(a, b);
        assert!(as_u128(d) == 0, 1);

        let a = from_u128(100);
        let b = from_u128(51);
        let d = mod(a, b);
        assert!(as_u128(d) == 49, 2);
    }

    #[test_only]
    use self::u256::shl_u8;

    #[test]
    fun test_shift_left() {
        let a = from_u128(100);
        let b = shl_u8(a, 2);

        assert!(as_u128(b) == 400, 0);
    }

    #[test_only]
    use self::u256::bits;

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

    #[test_only]
    use self::u256::byte;

    #[test]
    fun test_byte() {
        let a = from_string(&b"12387123871231238728283172387");

        assert!(byte(from_u128(20), a) == from_u128(40), 0);
        assert!(byte(from_u128(25), a) == from_u128(15), 1);
        assert!(byte(from_u128(30), a) == from_u128(174), 2);

        let m = (U64_MAX as u64);
        let max = new_u256(m, m, m, m);

        assert!(byte(from_u128(2), max) == from_u128(255), 3);
        assert!(byte(from_u128(1), max) == from_u128(255), 4);
        assert!(byte(from_u128(0), max) == from_u128(255), 5);
        assert!(byte(from_u128(17), max) == from_u128(255), 6);
    }

    #[test_only]
    use self::u256::compare;

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

    #[test_only]
    use self::u256::exp;

    #[test]
    fun test_exp() {
        let a = from_u128(2);
        let b = from_u128(3);
        let c = exp(a, b);
        assert!(c == from_u128(8), 0);

        let a = from_u128(0);
        let b = from_u128(0);
        let c = exp(a, b);
        assert!(c == from_u128(1), 1);

        let a = from_u128(83);
        let b = from_u128(13);
        let c = exp(a, b);
        assert!(c == from_string(&b"8871870642308873326043363"), 2);
    }

    #[test_only]
    use self::u256::new_u256;

    #[test]
    fun test_get() {
        let a = new_u256(1, 2, 3, 4);

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

    #[test_only]
    use self::u256::put;

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

    #[test_only]
    use self::u256::{eq, one, ne};

    #[test]
    fun test_one() {
        let a = one();
        assert!(eq(a, from_u128(1)), 0);
    }

    #[test]
    fun test_eq() {
        assert!(eq(from_u128(2927), from_string(&b"2927")), 0);
        assert!(from_u128(2927) == from_string(&b"2927"), 1);
        assert!(get_negative(from_u128(10)) == from_string(&b"-10"), 2);
    }

    #[test]
    fun test_ne() {
        assert!(!(ne(from_u128(2927), from_string(&b"2927"))), 0);
        assert!(!(from_u128(2927) != from_string(&b"2927")), 1);
        assert!(!(get_negative(from_u128(10)) != from_string(&b"-10")), 2);
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

    #[test_only]
    use self::u256::from_string;

    #[test]
    fun test_from_string() {
        let num = from_string(&b"1313539323434");
        assert!(as_u128(num) == 1313539323434, 0);

        let num = from_string(&b"231238729");
        assert!(as_u128(num) == 231238729, 1);

        let num = from_string(&b"-231238729");
        assert!(as_u128(get_negative(num)) == 231238729, 2);

        let num = from_string(&b"0");
        assert!(as_u128(num) == 0, 3);

        let num = from_string(&b"-1");
        assert!(as_u128(get_negative(num)) == 1, 3);
    }

    #[test_only]
    use self::u256::overflowing_add;

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
        let a = new_u256(max, max, max, max);

        let s = as_u128(overflowing_add(a, from_u128(1)));
        assert!(s == 0, 1);

        s = as_u128(overflowing_add(a, from_u128(2)));
        assert!(s == 1, 2);

        let c = compare(&overflowing_add(a, a), &a);
        assert!(c == 1, 3);

        s = as_u128(
            overflowing_add(overflowing_add(a, a), from_u128(2))
        );
        assert!(s == 0, 4);
    }

    #[test_only]
    use self::u256::overflowing_sub;

    #[test]
    fun test_sub() {
        let max = (U64_MAX as u64);
        let a = new_u256(max, max, max, max);

        let s = as_u128(overflowing_sub(from_u128(0), a));
        assert!(s == 1, 1);

        s = as_u128(overflowing_sub(from_u128(1), a));
        assert!(s == 2, 2);

        let c = compare(&overflowing_sub(a, a), &from_u128(0));
        assert!(c == 0, 3);

        c = compare(
            &overflowing_sub(
                overflowing_sub(from_u128(0), a),
                from_u128(2)
            ),
            &a
        );
        assert!(c == 0, 4);
    }


    #[test_only]
    use self::u256::overflowing_mul;

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
        let a = new_u256(max, max, max, max);

        let res = overflowing_mul(a, from_u128(2));
        assert!(compare(&res, &from_string(&b"-2")) == 0, 0);

        let res = overflowing_mul(a, a);
        assert!(compare(&res, &from_string(&b"1")) == 0, 0);
    }

    #[test_only]
    use self::u256::add_mod;

    #[test]
    fun test_add_mod() {
        let a = from_string(&b"1000000000");
        let b = from_string(&b"1");
        let mod = from_string(&b"1000000000");
        let c = add_mod(a, b, mod);

        assert!(as_u128(c) == 1, 0);

        let a = from_string(&b"5");
        let b = from_string(&b"3");
        let mod = from_string(&b"9");
        let c = add_mod(a, b, mod);

        assert!(as_u128(c) == 8, 1);

        let max = from_string(&b"115792089237316195423570985008687907853269984665640564039457584007913129639935");
        let a = max;
        let b = max;
        let mod = from_string(&b"19");
        let c = add_mod(a, b, mod);

        assert!(as_u128(c) == 11, 2);

        let max = from_string(&b"115792089237316195423570985008687907853269984665640564039457584007913129639935");
        let a = max;
        let b = max;
        let mod = from_string(&b"9973");
        let c = add_mod(a, b, mod);

        assert!(as_u128(c) == 1594, 3);
    }

    #[test]
    #[expected_failure(abort_code = 2)]
    fun test_add_mod_abort() {
        let a = from_string(&b"1000000000");
        let b = from_string(&b"1");
        let mod = from_string(&b"0");
        let _ = add_mod(a, b, mod);
    }

    #[test_only]
    use self::u256::mul_mod;

    #[test]
    fun test_mul_mod() {
        let a = from_string(&b"1000000000");
        let b = from_string(&b"1");
        let mod = from_string(&b"1000000000");
        let c = mul_mod(a, b, mod);

        assert!(as_u128(c) == 0, 0);

        let a = from_string(&b"5");
        let b = from_string(&b"3");
        let mod = from_string(&b"9");
        let c = mul_mod(a, b, mod);

        assert!(as_u128(c) == 6, 1);

        let max = from_string(&b"115792089237316195423570985008687907853269984665640564039457584007913129639935");
        let a = max;
        let b = max;
        let mod = from_string(&b"19");
        let c = mul_mod(a, b, mod);

        assert!(as_u128(c) == 16, 2);

        let max = from_string(&b"115792089237316195423570985008687907853269984665640564039457584007913129639935");
        let a = max;
        let b = max;
        let mod = from_string(&b"9973");
        let c = mul_mod(a, b, mod);

        assert!(as_u128(c) == 6910, 3);
    }

    #[test_only]
    use self::u256::{from_bytes, to_bytes, write, from_signer};

    #[test(self = @0xffeeddccbbaa99887766554433221100f0e0d0c0b0a090807060504030201000)]
    fun test_encode_decode(self: &signer) {
        let addr = from_signer(self);
        let bytes = to_bytes(&addr);
        let val = *std::vector::borrow(&bytes, 7);
        assert!(from_bytes(&bytes, from_u128(0)) == addr, (val as u64));
        write(&addr, &mut bytes, 0);
        assert!(from_bytes(&bytes, from_u128(0)) == addr, 1);
    }

    #[test(self = @0x4213421342134213)]
    fun test_address_to_u128_2(self: &signer) {
        assert!(from_signer(self) == from_u128(0x4213421342134213), 0);
        let addr = from_signer(self);
        let bytes = to_bytes(&addr);
        let val = *std::vector::borrow(&bytes, 7);
        assert!(from_bytes(&bytes, from_u128(0)) == addr, (val as u64));
        write(&addr, &mut bytes, 0);
        assert!(from_bytes(&bytes, from_u128(0)) == addr, (val as u64));
    }

    #[test(self = @0x42)]
    fun test_address_to_u128_1(self: &signer) {
        assert!(from_signer(self) == from_u128(0x42), 0);
        let addr = from_signer(self);
        let bytes = to_bytes(&addr);
        let val = *std::vector::borrow(&bytes, 7);
        assert!(from_bytes(&bytes, from_u128(0)) == addr, (val as u64));
        write(&addr, &mut bytes, 0);
        assert!(from_bytes(&bytes, from_u128(0)) == addr, (val as u64));
    }

    #[test_only]
    use self::u256::signextend;

    #[test]
    fun test_signextend() {
        let a = from_u128(10);
        let b = from_u128(2);
        let c = signextend(a, b);
        assert!(c == from_u128(2), 0);

        let a = from_u128(15);
        let b = from_string(&b"-531323222");
        let c = signextend(a, b);
        assert!(c == from_string(&b"-531323222"), 1);
    }

    #[test_only]
    use self::u256::sar;

    #[test]
    fun test_signed_shift_right() {
        let a = from_u128(100);
        let b = sar(a, from_u128(2));
        assert!(as_u128(b) == 25, 0);

        let a = from_u128(2472);
        let b = sar(a, from_u128(4));
        assert!(as_u128(b) == 154, 1);

        let a = from_string(&b"-200");
        let b = sar(a, from_u128(2));
        assert!(b == from_string(&b"-50"), 2);

        let a = from_string(&b"-200");
        let b = sar(a, from_u128(17));
        assert!(b == from_string(&b"-1"), 3);


        let a = from_string(&b"0");
        let b = sar(a, from_u128(17));
        assert!(b == from_string(&b"0"), 4);

        let a = from_string(&b"-20000000120482421");
        let b = sar(a, from_u128(300));
        assert!(b == from_string(&b"-1"), 5);

        let a = from_string(&b"200");
        let b = sar(a, from_u128(300));
        assert!(b == from_string(&b"0"), 6);
    }

}
