// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library Devide {
    // # devide_const_2()
    function devide_const_2() public pure returns (uint128) {
        return 4 / 2;
    }

    // # devide_const_0()
    function devide_const_0() public pure returns (uint128) {
        return 0 / 4;
    }

    // # devide_const_4()
    function devide_const_4() public pure returns (uint128) {
        uint128 a = 8;
        uint128 b = 2;

        return a / b;
    }
}
