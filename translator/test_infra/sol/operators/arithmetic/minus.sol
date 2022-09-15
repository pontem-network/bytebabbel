// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library Minus {
    // # minus_const_100()
    function minus_const_100() public pure returns (uint128) {
        uint128 a = 128;
        uint128 b = 28;
        return a - b;
    }

    // # minus_const_80()
    function minus_const_80() public pure returns (uint128) {
        uint128 a = 128;
        uint128 b = 28;
        uint128 c = a - b - 20;
        return c;
    }

    // # minus_const_50()
    function minus_const_50() public pure returns (uint128) {
        uint128 a = 128;
        uint128 b = 28;
        uint128 c = (a - b) - 50;
        a = a - c;
        return a - b;
    }

    // # minus_const_95()
    function minus_const_95() public pure returns (uint128) {
        return 100 - (10 - 5);
    }

    // # minusdivide_params_with_const(0)
    // # minusdivide_params_with_const(*)
    function minusdivide_params_with_const(uint128 a) public pure returns (uint128) {
        return a - 4 / 2;
    }

    // # minusdivide_params_with_const_2(0)
    // # minusdivide_params_with_const_2(*)
    function minusdivide_params_with_const_2(uint128 a) public pure returns (uint128) {
        return (a - 4) / 2;
    }


    // # minusdivide_params_with_const_3(4, 4, 2)
    function minusdivide_params_with_const_3(uint128 a, uint128 b, uint128 c) public pure returns (uint128) {
        return a - b / c;
    }

    // # minusdivide_params_with_const(0)
    // # minusdivide_params_with_const_4(*)
    function minusdivide_params_with_const_4(uint128 a) public pure returns (uint128) {
        uint128 b = 4;
        uint128 c = 2;
        return a - b / c;
    }
}
