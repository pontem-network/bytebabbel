// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library Devide {
    // # devide_const_2() 2
    function devide_const_2() public pure returns (uint) {
        return 4 / 2;
    }

    // # devide_const_0() 0
    function devide_const_0() public pure returns (uint) {
        return 0 / 4;
    }

    // # devide_const_4() 4
    function devide_const_4() public pure returns (uint) {
        uint a = 8;
        uint b = 2;

        return a / b;
    }
}
