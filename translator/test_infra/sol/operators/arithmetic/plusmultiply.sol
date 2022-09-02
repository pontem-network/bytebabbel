// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library PlusMultiply {
    // # plusmultiply_const_1() 7
    function plusmultiply_const_1() public pure returns (uint128) {
        return 1 + 2 * 3;
    }

    // # plusmultiply_const_2() 7
    function plusmultiply_const_2() public pure returns (uint128) {
        uint128 a = 1;
        uint128 b = 2;
        uint128 c = 3;
        return a + b * c;
    }

    // # plusmultiply_const_3() 7
    function plusmultiply_const_3() public pure returns (uint128) {
        uint128 a = 1;
        uint128 b = 2;
        uint128 c = 3;
        return a + (b * c);
    }

    // # plusmultiply_const_9() 9
    function plusmultiply_const_9() public pure returns (uint128) {
        uint128 a = 1;
        uint128 b = 2;
        uint128 c = 3;
        return (a + b) * c;
    }

    // # plusmultiply_params_with_const(1) 9
    function plusmultiply_params_with_const(uint128 a) public pure returns (uint128) {
        return (a + 2) * 3;
    }
}
