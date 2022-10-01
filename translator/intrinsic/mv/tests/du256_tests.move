module self::du256_tests {
    #[test_only]
    use self::du256::{new_du256, get_d, put_d};

    #[tets_only]
    use self::u256::{get, du256_to_u256};

    #[test]
    fun test_du256_to_u256() {
        let a = new_du256(255, 100, 50, 300, 0, 0, 0, 0);

        let (m, overflow) = du256_to_u256(a);
        assert!(!overflow, 0);
        assert!(get(&m, 0) == get_d(&a, 0), 1);
        assert!(get(&m, 1) == get_d(&a, 1), 2);
        assert!(get(&m, 2) == get_d(&a, 2), 3);
        assert!(get(&m, 3) == get_d(&a, 3), 4);

        put_d(&mut a, 4, 100);
        put_d(&mut a, 5, 5);

        let (m, overflow) = du256_to_u256(a);
        assert!(overflow, 5);
        assert!(get(&m, 0) == get_d(&a, 0), 6);
        assert!(get(&m, 1) == get_d(&a, 1), 7);
        assert!(get(&m, 2) == get_d(&a, 2), 8);
        assert!(get(&m, 3) == get_d(&a, 3), 9);
    }

    #[test]
    fun test_get_d() {
        let a = new_du256(1, 2, 3, 4, 5, 6, 7, 8);

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
        let a = new_du256(1, 2, 3, 4, 5, 6, 7, 8);

        get_d(&a, 8);
    }

    #[test]
    fun test_put_d() {
        let a = new_du256(1, 2, 3, 4, 5, 6, 7, 8);

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
        let a = new_du256(1, 2, 3, 4, 5, 6, 7, 8);

        put_d(&mut a, 8, 0);
    }
}