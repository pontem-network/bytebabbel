#[test_only]
module self::mem_tests {
    use self::mem;

    #[test]
    fun load_store_with_same_offset() {
        let memory = mem::new(1024);

        mem::mstore(&mut memory, 0, 0x42);
        let val = mem::mload(&mut memory, 0);
        assert!(val == 0x42, 0);
        assert!(mem::effective_len(&memory) == 16, 2);


        mem::mstore(&mut memory, 1, 340282366920938463463374607431768211455);
        let val = mem::mload(&mut memory, 1);
        assert!(val == 340282366920938463463374607431768211455, 1);
        assert!(mem::effective_len(&memory) == 32, 2);
    }

    #[test]
    fun load_store_loop() {
        let memory = mem::new(2048);

        let offset = 0;
        while (offset < 1024) {
            mem::mstore(&mut memory, offset, offset);
            offset = offset + 16;
        };

        let offset = 0;
        while (offset < 1024) {
            let val = mem::mload(&mut memory, offset);
            assert!(val == offset, 1);
            offset = offset + 16;
        };
    }

    #[test]
    fun mem_shift() {
        let memory = mem::new(1024);
        mem::mstore(&mut memory, 0, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
        let val = mem::mload(&mut memory, 8);
        assert!(val == 0xFFFFFFFFFFFFFFFF0000000000000000, 0);
    }

    #[test]
    fun mem_store_8() {
        let memory = mem::new(1024);

        mem::mstore8(&mut memory, 0, 0xAA);
        let val = mem::mload(&mut memory, 0);
        assert!(val == 0xAA000000000000000000000000000000, 1);

        mem::mstore8(&mut memory, 1, 0xFF);
        let val = mem::mload(&mut memory, 0);
        assert!(val == 0xAAFF0000000000000000000000000000, 2);

        mem::mstore8(&mut memory, 2, 0x11);
        let val = mem::mload(&mut memory, 0);
        assert!(val == 0xAAFF1100000000000000000000000000, 3);

        mem::mstore8(&mut memory, 15, 0xCC);
        let val = mem::mload(&mut memory, 0);
        assert!(val == 0xAAFF11000000000000000000000000CC, (val as u64));
    }

    #[test]
    #[expected_failure(abort_code = 1)]
    fun init_zero_size() {
        mem::new(0);
    }

    #[test]
    #[expected_failure(abort_code = 1)]
    fun init_out_of_limit() {
        mem::new(17 * 1014);
    }
}
