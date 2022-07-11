// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library MathFn {
    function summation_3() public pure returns (uint) {
        uint a = 1;
        uint b = 2;
        return a + b;
    }

    function summation_15() public pure returns (uint) {
        uint a = 1 + 3 + 4;
        uint b = 2;
        return a + b + 5;
    }

    function summation_30() public pure returns (uint) {
        uint a = 1;
        uint b = 2;
        uint c = (a + b) + (3 + 4 + 5);
        c = c + 5;
        uint d = 10;
        return d + c;
    }

    function subtraction_100() public pure returns (uint) {
        uint a = 128;
        uint b = 28;
        return a - b;
    }

    function subtraction_50() public pure returns (uint) {
        uint a = 128;
        uint b = 28;
        uint c = (a - b) - 50;
        a = a - c;
        return a - b;
    }

    function subsum_106() public pure returns (uint) {
        uint a = 128 - 12;
        uint b = 28 + 12;
        uint c = 30;
        return a - b + c;
    }

    function subsum_40() public pure returns (uint) {
        uint a = 10;
        uint b = 20;
        uint c = 50 - 20;
        a = a + (c - b);
        return a + a;
    }
}
