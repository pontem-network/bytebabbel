// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library Simple {
    // # plus(1,2,3)
    // # plus(18446744073709551615,0,0)
    // # plus(18446744073709551615,18446744073709551615,18446744073709551615)
    // # plus(340282366920938463463374607431768211455,0,0)
    // # plus(340282366920938463463374607431768211455,1,0)
    function plus(uint128 a, uint128 b, uint128 c) public pure returns (uint128) {
        return a + b + c;
    }

    // # minus(3,1,1) 1
    // # minus(18446744073709551615,18446744073709551615,0)
    // # minus(18446744073709551615,0,1)
    // # minus(340282366920938463463374607431768211455,340282366920938463463374607431768211455,0)
    // # minus(340282366920938463463374607431768211455,0,0)
    // # minus(340282366920938463463374607431768211455,1,1)
    // # minus(0,0,1)
    function minus(uint128 a, uint128 b, uint128 c) public pure returns (uint128) {
        return a - b - c;
    }

    // # devide(2,1,1)
    // # devide(12,4,3)
    // # devide(4,2,1)
    // # devide(4,3,1)
    // # devide(0,3,1)
    // # devide(3,0,1)
    function devide(uint128 a, uint128 b, uint128 c) public pure returns (uint128) {
        return a / b / c;
    }

    // # multiply(1,2,3)
    // # multiply(18446744073709551615,1,1)
    // # multiply(18446744073709551615,2,10)
    // # multiply(340282366920938463463374607431768211455,1,1)
    // # multiply(340282366920938463463374607431768211455,2,1)
    function multiply(uint128 a, uint128 b, uint128 c) public pure returns (uint128) {
        return a * b * c;
    }

    // # dec(2)
    // # dec(255)
    // # dec(0)
    function dec(uint128 a) public pure returns (uint128) {
        return --a;
    }

    // # dec_s(255)
    function dec_s(uint128 a) public pure returns (uint128) {
        return 10 + --a;
    }

    // # mod(9,5)
    function mod(uint128 a, uint128 b) public pure returns (uint128) {
        return a % b;
    }

    // # inc(2)
    function inc(uint128 a) public pure returns (uint128) {
        return ++a;
    }
}
