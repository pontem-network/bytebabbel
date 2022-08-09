// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library MathMultiply {
    // # multiply_const_2() 2
    function multiply_const_2() public pure returns (uint) {
        uint a = 1;
        uint b = 2;
        return a * b;
    }

    // # multiply_const_0() 0
    function multiply_const_0() public pure returns (uint) {
        return 0 * 100;
    }

    // # multiply_const_99() 99
    function multiply_const_99() public pure returns (uint) {
        return 1 * 99;
    }

    // # multiply_const_3() 3
    function multiply_const_3() public pure returns (uint) {
        uint a = 1;
        uint b = 3;
        uint c = a * b;
        return c;
    }

    // # multiply_const_120() 120
    function multiply_const_120() public pure returns (uint) {
        uint a = 1 * 3 * 4;
        uint b = 2;
        return a * b * 5;
    }

    // # multiply_const_6000() 6000
    function multiply_const_6000() public pure returns (uint) {
        uint a = 1;
        uint b = 2;
        uint c = (a * b) * (3 * 4 * 5);
        c = c * 5;
        uint d = 10;
        return d * c;
    }
}
