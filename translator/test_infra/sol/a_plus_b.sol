// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

contract APlusB {
    function plus() public pure returns (uint128) {
        return plus_1(13, 14);
    }

    function plus_1(uint128 a, uint128 b) internal pure returns (uint128) {
        return a + b;
    }

    function minus() public pure returns (uint128) {
        return 14 - 13;
    }
}
