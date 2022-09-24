module self::test_u256 {
    #[test_only]
    use self::u256::{read_u64};

    #[test_only]
    use std::vector;

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

}