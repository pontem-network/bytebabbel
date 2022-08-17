// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

contract Simple {
    // # or_uint (20, 10) 30
    function or_uint(uint a, uint b) public pure returns (uint) {
        return a | b;
    }

    // # xor_uint (20, 10) 30
    function xor_uint(uint a, uint b) public pure returns (uint) {
        return a ^ b;
    }


    // # leftshift_uint (20, 10) 20480
    function leftshift_uint(uint a, uint b) public pure returns (uint) {
        return a << b;
    }


    // # rightshift_uint (20, 10) 0
    function rightshift_uint(uint a, uint b) public pure returns (uint) {
        return a >> b;
    }


    // # and_uint (20, 10) 0
    // # and_uint (20, 20) 20
    function and_uint(uint a, uint b) public pure returns (uint) {
        return a & b;
    }
}
