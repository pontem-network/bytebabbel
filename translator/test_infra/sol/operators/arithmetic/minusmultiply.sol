// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library MinusMultiply {

    // # minusmultiply_const_2()
    function minusmultiply_const_2() public pure returns (uint128) {
        return 8 - 2 * 3;
    }

    // # minusmultiply_const_11()
    function minusmultiply_const_11() public pure returns (uint128) {
        return 20 - (3 * 3);
    }

    // # minusmultiply_const_18()
    function minusmultiply_const_18() public pure returns (uint128) {
        return (8 - 2) * 3;
    }

    // # minusmultiply_const_0()
    function minusmultiply_const_0() public pure returns (uint128) {
        return (8 - 8) * 3;
    }

    // # minusmultiply_params_with_const(6)
    // # minusmultiply_params_with_const(7)
    // # ignore_minusmultiply_params_with_const(0)
    function minusmultiply_params_with_const(uint128 a) public pure returns (uint128) {
        uint128 b = 2;
        uint128 c = 3;
        return a - b * c;
    }

    // # minusmultiply_params_with_const_2(6)
    function minusmultiply_params_with_const_2(uint128 a) public pure returns (uint128) {
        uint128 b = 2;
        uint128 c = 3;
        return (a - b) * c;
    }

    // # minusmultiply_params(0,0,0)
    // # minusmultiply_params(8,2,3)
    function minusmultiply_params(uint128 a, uint128 b, uint128 c) public pure returns (uint128) {
        return a - b * c;
    }
}
