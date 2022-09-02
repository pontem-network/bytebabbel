// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

contract Simple {

    // # plus (20) 30
    function plus(uint128 a) public pure returns (uint128) {
        a += 10;
        return a;
    }

    // # minus (20) 10
    function minus(uint128 a) public pure returns (uint128) {
        a -= 10;
        return a;
    }

    // # divede (200) 20
    function divede(uint128 a) public pure returns (uint128) {
        a /= 10;
        return a;
    }

    // # multiply (10) 100
    function multiply(uint128 a) public pure returns (uint128) {
        a *= 10;
        return a;
    }

    // # and_uint128 (20) 20
    // # and_uint128 (10) 0
    function and_uint(uint128 a) public pure returns (uint128) {
        a &= 20;
        return a;
    }

    // # mod (12) 2
    function mod(uint128 a) public pure returns (uint128) {
        a %= 10;
        return a;
    }

    // # or_uint128 (20) 30
    function or_uint(uint128 a) public pure returns (uint128) {
        a |= 10;
        return a;
    }

    // # xor_uint128 (20) 30
    function xor_uint(uint128 a) public pure returns (uint128) {
        a ^= 10;
        return a;
    }

    // # leftshift_uint128 (20) 20480
    function leftshift_uint(uint128 a) public pure returns (uint128) {
        a <<= 10;
        return a;
    }

    // # rightshift_uint128 (20) 0
    function rightshift_uint(uint128 a) public pure returns (uint128) {
        a >>= 10;
        return a;
    }
}
