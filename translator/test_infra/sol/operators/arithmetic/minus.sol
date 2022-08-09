// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library Minus {
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

    // # minusdivide_params_with_const(4) 2
    // # minusdivide_params_with_const(340282366920938463463374607431768211455) 340282366920938463463374607431768211453
    // # minusdivide_params_with_const(0) !panic
    function minusdivide_params_with_const(uint a) public pure returns (uint) {
        return a - 4 / 2;
    }

    // # minusdivide_params_with_const_2(6) 1
    function minusdivide_params_with_const_2(uint a) public pure returns (uint) {
        return (a - 4) / 2;
    }


    // # minusdivide_params_with_const_3(4, 4, 2) 2
    function minusdivide_params_with_const_3(uint a, uint b, uint c) public pure returns (uint) {
        return a - b / c;
    }

    // # minusdivide_params_with_const_4(4) 2
    function minusdivide_params_with_const_4(uint a) public pure returns (uint) {
        uint b = 4;
        uint c = 2;
        return a - b / c;
    }
}
