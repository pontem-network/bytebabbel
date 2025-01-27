module self::memory_tests {
    #[test_only]
    use self::u256::{from_u128, as_u128, U256, from_bytes, zero, eq};

    #[test_only]
    use self::memory::{request_buffer_len, code_copy, get_data};

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
        let len = as_u128(request_buffer_len(&buff));
        assert!(len == 7 + 4, 1);
    }

    #[test_only]
    use self::memory::{new_mem, mload, mstore};

    #[test]
    fun test_random_access() {
        let memory = new_mem(1024);
        mstore(&mut memory, from_u128(64), from_u128(128));
        let value = mload(&mut memory, from_u128(64));
        let value = as_u128(value);
        assert!(value == 128, 1);
        let value = mload(&mut memory, from_u128(0));
        let value = as_u128(value);
        assert!(value == 0, 2);

        let value = mload(&mut memory, from_u128(32));
        let value = as_u128(value);
        assert!(value == 0, 2);
    }

    #[test_only]
    use self::memory::effective_len;

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
        mstore(&mut memory, from_u128(0), from_u128(0x0000000000000000FFFFFFFFFFFFFFFF));
        let val = mload(&mut memory, from_u128(8));
        assert!(as_u128(val) == 0xFFFFFFFFFFFFFFFF0000000000000000, 0);
    }

    #[test_only]
    fun read_signer(addr: &signer): U256 {
        let encoded = std::bcs::to_bytes(addr);
        from_bytes(&encoded, zero())
    }

    #[test_only]
    use self::memory::mstore8;

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
        let expected = read_signer(v1);
        assert!(eq(val, expected), 1);

        let val = mload(&mut memory, from_u128(1));
        assert!(eq(val, zero()), 1);

        mstore8(&mut memory, from_u128(1), from_u128(0xFF));
        let val = mload(&mut memory, from_u128(0));
        let expected = read_signer(v2);
        assert!(eq(val, expected), 2);

        mstore8(&mut memory, from_u128(2), from_u128(0x11));
        let val = mload(&mut memory, from_u128(0));
        let expected = read_signer(v3);
        assert!(eq(val, expected), 3);

        mstore8(&mut memory, from_u128(8), from_u128(0x11));
        mstore8(&mut memory, from_u128(7), from_u128(0x22));
        let val = mload(&mut memory, from_u128(0));
        let expected = read_signer(v4);
        assert!(eq(val, expected), 4);

        mstore8(&mut memory, from_u128(15), from_u128(0x33));
        mstore8(&mut memory, from_u128(16), from_u128(0x44));
        mstore8(&mut memory, from_u128(23), from_u128(0x55));
        mstore8(&mut memory, from_u128(24), from_u128(0x66));
        mstore8(&mut memory, from_u128(31), from_u128(0x77));

        let val = mload(&mut memory, from_u128(0));
        let expected = read_signer(v5);
        assert!(eq(val, expected), 5);
    }

    #[test_only]
    use self::memory::hash;

    #[test(
        s1 = @0x6261be5de65349dedcf98dad3041f331b4a397546079ef17542df4fbbf359787,
        s2 = @0x81dbecc6aee62dfb2250aeca0ca406d6dc61788004aaf116fa9c2c61d00a5897,
        hash_1 = @0xb5553de315e0edf504d9150af82dafa5c4667fa618ed0a6f19c69b41166c5510,
        hash_2 = @0x177c65462e6af6bb2b626ead9d5ce6b98ec801e505e39db4a6d2a19881207143,
        hash_3 = @0xec608c46eab7d1b8ece1573a1436f4e5506c146cf372fb15e656cc768a6ccac0,
        hash_4 = @0x3463f25cc43520278476551251fefecc2df492f39142f907aabafdd31d7bf365,
        hash_5 = @0x5037e1a5e02e081b1b850b130eca7ac17335fdf4c61cc5ff6ae765196fb0d5b3,
    )]
    fun test_hash(s1: &signer, s2: &signer, hash_1: &signer, hash_2: &signer, hash_3: &signer, hash_4: &signer, hash_5: &signer) {
        let memory = new_mem(1024);

        mstore(&mut memory, from_u128(0), read_signer(s1));
        mstore(&mut memory, from_u128(32), read_signer(s2));

        let resp = hash(&mut memory, from_u128(0), from_u128(1));
        assert!(eq(resp, read_signer(hash_1)), 1);

        let resp = hash(&mut memory, from_u128(0), from_u128(32));
        assert!(eq(resp, read_signer(hash_2)), 2);

        let resp = hash(&mut memory, from_u128(0), from_u128(64));
        assert!(eq(resp, read_signer(hash_3)), 3);

        let resp = hash(&mut memory, from_u128(0), from_u128(70));
        assert!(eq(resp, read_signer(hash_4)), 4);

        let resp = hash(&mut memory, from_u128(64), from_u128(6));
        assert!(eq(resp, read_signer(hash_5)), 5);
    }

    #[test]
    #[expected_failure]
    fun init_zero_size() {
        new_mem(0);
    }

    #[test]
    fun test_code_copy() {
        let memory = new_mem(1024);
        let code = b"This is the large string that we are testing. And it is bigger than 32 bytes.";
        code_copy(&mut memory, from_u128(0), code);
        let mem = get_data(&memory);
        assert!(mem == &code, 1);

        let code2 = b"Never ask strings for their size....";
        code_copy(&mut memory, from_u128(46), code2);
        let mem = get_data(&memory);
        assert!(mem == &b"This is the large string that we are testing. Never ask strings for their size....", 2);

        let code3 = b"ps";
        code_copy(&mut memory, from_u128(100), code3);

        let mem = get_data(&memory);
        assert!(mem == &b"This is the large string that we are testing. Never ask strings for their size....\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0ps", 3);
    }
}
