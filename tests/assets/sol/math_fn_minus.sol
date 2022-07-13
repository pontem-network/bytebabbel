// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library MathMinusFn {
    // # minus_const_100() 100
    function minus_const_100() public pure returns (uint) {
        uint a = 128;
        uint b = 28;
        return a - b;
    }

    // # minus_const_80() 80
    function minus_const_80() public pure returns (uint) {
        uint a = 128;
        uint b = 28;
        uint c = a - b - 20;
        return c;
    }

    // # minus_const_50() 50
    function minus_const_50() public pure returns (uint) {
        uint a = 128;
        uint b = 28;
        uint c = (a - b) - 50;
        a = a - c;
        return a - b;
    }

    // # minus_const_95() 95
    function minus_const_95() public pure returns (uint) {
        return 100 - (10 - 5);
    }

    // # minus_params(2,1) 1
    // # minus_params(18446744073709551615,18446744073709551615) 0
    // # minus_params(18446744073709551615,0) 18446744073709551615
    // # minus_params(340282366920938463463374607431768211455,340282366920938463463374607431768211455) 0
    // # minus_params(340282366920938463463374607431768211455,0) 340282366920938463463374607431768211455
    // # minus_params(340282366920938463463374607431768211455,1) 340282366920938463463374607431768211454
    // # minus_params(0,1) !panic
    function minus_params(uint a, uint b) public pure returns (uint) {
        return a - b;
    }

    // # minusdivide_params_with_const(4) 2
    // # minusdivide_params_with_const(340282366920938463463374607431768211455) 340282366920938463463374607431768211453
    // # minusdivide_params_with_const(0) !panic
    function minusdivide_params_with_const(uint a) public pure returns (uint) {
        return a - 4 / 2;
    }
}
