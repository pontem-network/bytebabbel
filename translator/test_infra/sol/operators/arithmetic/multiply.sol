// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library MathMultiply {
    // # multiply_const_2()
    function multiply_const_2() public pure returns (uint128) {
        uint128 a = 1;
        uint128 b = 2;
        return a * b;
    }

    // # multiply_const_0()
    function multiply_const_0() public pure returns (uint128) {
        return 0 * 100;
    }

    // # multiply_const_99()
    function multiply_const_99() public pure returns (uint128) {
        return 1 * 99;
    }

    // # multiply_const_3()
    function multiply_const_3() public pure returns (uint128) {
        uint128 a = 1;
        uint128 b = 3;
        uint128 c = a * b;
        return c;
    }

    // # multiply_const_120()
    function multiply_const_120() public pure returns (uint128) {
        uint128 a = 1 * 3 * 4;
        uint128 b = 2;
        return a * b * 5;
    }

    // # multiply_const_6000()
    function multiply_const_6000() public pure returns (uint128) {
        uint128 a = 1;
        uint128 b = 2;
        uint128 c = (a * b) * (3 * 4 * 5);
        c = c * 5;
        uint128 d = 10;
        return d * c;
    }
}
