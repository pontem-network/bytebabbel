module self::memory_tests {
    #[test_only]
    use self::u256::{from_u128, as_u128, U256, from_bytes, zero, eq};

    #[test_only]
    use self::memory::{request_buffer_len};

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
        hash_1 = @0xb039179a8a4ce2c252aa6f2f25798251c19b75fc1508d9d511a191e0487d64a7,
        hash_2 = @0xcc636d0dc01c94023106c1459b926e17d25572e9cbf154ec7489c6264e83ec7d,
        hash_3 = @0x1cdfbfb2fdbcc015c6c45e292d5927b775f9d6595a0c9d3ff9c029e1fe2ff7f3,
        hash_4 = @0x5129046912a39ba87d481c3c8d8cd626cbfba7f089c9879d853d40ab63ab3775,
        hash_5 = @0xc1545e05e6777d834652396ad104e7e971a78d084a9b9df34f7a16fd493bf2b0,
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
    #[expected_failure(abort_code = 1)]
    fun init_zero_size() {
        new_mem(0);
    }
}