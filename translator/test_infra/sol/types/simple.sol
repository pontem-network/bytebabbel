// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

contract Simple {
    // # f_bool (true) true
    function f_bool(bool a) public pure returns (bool) {
        return a;
    }
//
//    // # f_uint (100_u256) 100_u256
//    function f_uint(uint a) public pure returns (uint) {
//        return a;
//    }
//
//    // # f_uint8 (100) 100
//    function f_uint8(uint8 a) public pure returns (uint8) {
//        return a;
//    }
//
//    // # f_uint16 (100) 100
//    function f_uint16(uint16 a) public pure returns (uint16) {
//        return a;
//    }
//
//    // # f_uint32 (100) 100
//    function f_uint32(uint32 a) public pure returns (uint32) {
//        return a;
//    }
//
//    // # f_uint64 (100) 100
//    function f_uint64(uint64 a) public pure returns (uint64) {
//        return a;
//    }
//
//    // # f_uint128 (100) 100
//    function f_uint128(uint128 a) public pure returns (uint128) {
//        return a;
//    }
//
//    // # f_uint256 (100) 100
//    function f_uint256(uint256 a) public pure returns (uint256) {
//        return a;
//    }
//
//    // # f_dec ()
//    function f_dec() public pure {
//        uint128 b = 2.5 + 0.5;
//        assert(b == 3);
//    }
//
//    // # f_auto_conv ()
//    function f_auto_conv() public pure {
//        uint8 y = 1;
//        uint16 z = 2;
//        uint32 x = y + z;
//        assert(x==3);
//    }
}
