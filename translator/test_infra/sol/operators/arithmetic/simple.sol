// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library Simple {
    // # plus(1,2,3) 6
    // # plus(18446744073709551615,0,0) 18446744073709551615
    // # plus(18446744073709551615,18446744073709551615,18446744073709551615) 55340232221128654845
    // # plus(340282366920938463463374607431768211455,0,0) 340282366920938463463374607431768211455
    // # plus(340282366920938463463374607431768211455,1,0) !panic
    function plus(uint a, uint b, uint c) public pure returns (uint) {
        return a + b + c;
    }

    // # minus(3,1,1) 1
    // # minus(18446744073709551615,18446744073709551615,0) 0
    // # minus(18446744073709551615,0,1) 18446744073709551614
    // # minus(340282366920938463463374607431768211455,340282366920938463463374607431768211455,0) 0
    // # minus(340282366920938463463374607431768211455,0,0) 340282366920938463463374607431768211455
    // # minus(340282366920938463463374607431768211455,1,1) 340282366920938463463374607431768211453
    // # minus(0,0,1) !panic
    function minus(uint a, uint b, uint c) public pure returns (uint) {
        return a - b - c;
    }

    // # devide(2,1,1) 2
    // # devide(12,4,3) 1
    // # devide(4,2,1) 2
    // # devide(4,3,1) 1
    // # devide(0,3,1) 0
    // # devide(3,0,1) !panic
    function devide(uint a, uint b, uint c) public pure returns (uint) {
        return a / b / c;
    }

    // # multiply(1,2,3) 6
    // # multiply(18446744073709551615,1,1) 18446744073709551615
    // # multiply(18446744073709551615,2,10) 368934881474191032300
    // # multiply(340282366920938463463374607431768211455,1,1) 340282366920938463463374607431768211455
    // # multiply(340282366920938463463374607431768211455,2,1) !panic
    function multiply(uint a, uint b, uint c) public pure returns (uint) {
        return a * b * c;
    }

    // # dec(2) 1
    // # dec(255) 254
    // # dec(0) !panic
    function dec(uint a) public pure returns (uint) {
        return --a;
    }

    // # dec_s(255) 264
    function dec_s(uint a) public pure returns (uint) {
        return 10 + --a;
    }


}
