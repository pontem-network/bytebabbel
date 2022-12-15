module self::u512_tests {
    /// Max `u64` value.
    const U64_MAX: u128 = 18446744073709551615;

    #[test_only]
    use self::u512::{new_u512, get_d, put_d};

    #[tets_only]
    use self::u256::{get, u512_to_u256, u256_to_u512, from_u128, as_u128};

    #[test]
    fun test_u512_to_u256() {
        let a = new_u512(255, 100, 50, 300, 0, 0, 0, 0);

        let (m, overflow) = u512_to_u256(a);
        assert!(!overflow, 0);
        assert!(get(&m, 0) == get_d(&a, 0), 1);
        assert!(get(&m, 1) == get_d(&a, 1), 2);
        assert!(get(&m, 2) == get_d(&a, 2), 3);
        assert!(get(&m, 3) == get_d(&a, 3), 4);

        put_d(&mut a, 4, 100);
        put_d(&mut a, 5, 5);

        let (m, overflow) = u512_to_u256(a);
        assert!(overflow, 5);
        assert!(get(&m, 0) == get_d(&a, 0), 6);
        assert!(get(&m, 1) == get_d(&a, 1), 7);
        assert!(get(&m, 2) == get_d(&a, 2), 8);
        assert!(get(&m, 3) == get_d(&a, 3), 9);
    }

    #[test]
    fun test_get_d() {
        let a = new_u512(1, 2, 3, 4, 5, 6, 7, 8);

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
    #[expected_failure]
    fun test_get_d_overflow() {
        let a = new_u512(1, 2, 3, 4, 5, 6, 7, 8);

        get_d(&a, 8);
    }

    #[test]
    fun test_put_d() {
        let a = new_u512(1, 2, 3, 4, 5, 6, 7, 8);

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
    #[expected_failure]
    fun test_put_d_overflow() {
        let a = new_u512(1, 2, 3, 4, 5, 6, 7, 8);

        put_d(&mut a, 8, 0);
    }

    #[test_only]
    use self::u512::compare_d;

    #[test]
    fun test_compare() {
        let a = u256_to_u512(&from_u128(1000));
        let b = u256_to_u512(&from_u128(50));

        let cmp = compare_d(&a, &b);
        assert!(cmp == 2, 0);

        a = u256_to_u512(&from_u128(100));
        b = u256_to_u512(&from_u128(100));
        cmp = compare_d(&a, &b);

        assert!(cmp == 0, 1);

        a = u256_to_u512(&from_u128(50));
        b = u256_to_u512(&from_u128(75));

        cmp = compare_d(&a, &b);
        assert!(cmp == 1, 2);
    }


    #[test_only]
    use self::u512::overflowing_add_d;

    #[test]
    fun test_add_d() {
        let a = new_u512(1, 0, 0, 0, 0, 0, 0, 0);
        let b = new_u512(1, 0, 0, 0, 0, 0, 0, 0);
        let c = overflowing_add_d(a, b);

        assert!(get_d(&c, 0) == 2, 0);

        let a = new_u512((U64_MAX as u64), (U64_MAX as u64), (U64_MAX as u64), (U64_MAX as u64), 0, 0, 0, 0);
        let b = new_u512((U64_MAX as u64), (U64_MAX as u64), (U64_MAX as u64), (U64_MAX as u64), 0, 0, 0, 0);
        let c = overflowing_add_d(a, b);
        let d = overflowing_add_d(c, new_u512(2, 0, 0, 0, 0, 0, 0, 0));

        assert!(get_d(&d, 4) == 2, 1);
    }

    #[test_only]
    use self::u512::overflowing_sub_d;

    #[test]
    fun test_sub_d() {
        let max = (U64_MAX as u64);
        let a = new_u512(max, max, max, max, max, max, max, max);

        let s = overflowing_sub_d(u256_to_u512(&from_u128(0)), a);
        let (s, _) = u512_to_u256(s);
        let s = as_u128(s);
        assert!(s == 1, 1);

        let (s, _) = u512_to_u256(
            overflowing_sub_d(u256_to_u512(&from_u128(1)), a)
        );
        assert!(as_u128(s) == 2, 2);

        let c = compare_d(&overflowing_sub_d(a, a), &u256_to_u512(&from_u128(0)));
        assert!(c == 0, 3);

        c = compare_d(
            &overflowing_sub_d(
                overflowing_sub_d(u256_to_u512(&from_u128(0)), a),
                u256_to_u512(&from_u128(2))
            ),
            &a
        );
        assert!(c == 0, 4);
    }

    #[test_only]
    use self::u512::mod_d;

    #[test]
    fun test_mod_d() {
        let a = from_u128(100);
        let b = from_u128(5);
        let d = mod_d(u256_to_u512(&a), u256_to_u512(&b));
        let (d, _) = u512_to_u256(d);
        assert!(as_u128(d) == 0, 0);

        let a = from_u128(100);
        let b = from_u128(0);
        let d = mod_d(u256_to_u512(&a), u256_to_u512(&b));
        let (d, _) = u512_to_u256(d);
        assert!(as_u128(d) == 0, 1);

        let a = from_u128(100);
        let b = from_u128(51);
        let d = mod_d(u256_to_u512(&a), u256_to_u512(&b));
        let (d, _) = u512_to_u256(d);
        assert!(as_u128(d) == 49, 2);
    }

    #[test_only]
    use self::u512::shl_u8_d;

    #[test]
    fun test_shift_left_d() {
        let a = u256_to_u512(&from_u128(100));
        let b = shl_u8_d(a, 2);
        let (b, _) = u512_to_u256(b);

        assert!(as_u128(b) == 400, 0);
    }

    #[test_only]
    use self::u512::shr_u8_d;

    #[test]
    fun test_shift_right_d() {
        let a = u256_to_u512(&from_u128(100));
        let b = shr_u8_d(a, 2);
        let (b, _) = u512_to_u256(b);

        assert!(as_u128(b) == 25, 0);
    }
    #[test_only]
    use self::u512::bits_d;

    #[test]
    fun test_bits_d() {
        let a = bits_d(&u256_to_u512(&from_u128(0)));
        assert!(a == 0, 0);

        a = bits_d(&u256_to_u512(&from_u128(255)));
        assert!(a == 8, 1);

        a = bits_d(&u256_to_u512(&from_u128(256)));
        assert!(a == 9, 2);

        a = bits_d(&u256_to_u512(&from_u128(300)));
        assert!(a == 9, 3);

        a = bits_d(&u256_to_u512(&from_u128(60000)));
        assert!(a == 16, 4);

        a = bits_d(&u256_to_u512(&from_u128(70000)));
        assert!(a == 17, 5);

        let b = u256_to_u512(&from_u128(70000));
        let sh = shl_u8_d(b, 100);
        assert!(bits_d(&sh) == 117, 6);

        let sh = shl_u8_d(sh, 100);
        assert!(bits_d(&sh) == 217, 7);

        let sh = shl_u8_d(sh, 308);
        assert!(bits_d(&sh) == 0, 8);
    }

}