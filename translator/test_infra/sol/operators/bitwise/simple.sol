// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

contract Simple {
    // # or_uint128 (20, 10) 30
    function or_uint(uint128 a, uint128 b) public pure returns (uint128) {
        return a | b;
    }

    // # xor_uint128 (20, 10) 30
    function xor_uint(uint128 a, uint128 b) public pure returns (uint128) {
        return a ^ b;
    }


    // # leftshift_uint128 (20, 10) 20480
    function leftshift_uint(uint128 a, uint128 b) public pure returns (uint128) {
        return a << b;
    }


    // # rightshift_uint128 (20, 10) 0
    function rightshift_uint(uint128 a, uint128 b) public pure returns (uint128) {
        return a >> b;
    }


    // # and_uint128 (20, 10) 0
    // # and_uint128 (20, 20) 20
    function and_uint(uint128 a, uint128 b) public pure returns (uint128) {
        return a & b;
    }
}
