// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

contract Simple {

    // # and_uint128 (20, 10) 0
    // # and_uint128 (20, 20) 20
    function and_uint(uint128 a, uint128 b) public pure returns (uint128) {
        return a & b;
    }
}
