// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

contract Simple {
    // # mod (12) 2
    function mod(uint a) public pure returns (uint) {
        a %= 10;
        return a;
    }

    // # or_uint (20) 30
    function or_uint(uint a) public pure returns (uint) {
        a |= 10;
        return a;
    }

    // # xor_uint (20) 30
    function xor_uint(uint a) public pure returns (uint) {
        a ^= 10;
        return a;
    }

    // # leftshift_uint (20) 20480
    function leftshift_uint(uint a) public pure returns (uint) {
        a <<= 10;
        return a;
    }

    // # rightshift_uint (20) 0
    function rightshift_uint(uint a, uint b) public pure returns (uint) {
        a >> 10;
        return a;
    }
}
