// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library MathMinusMultiplyFn {

    // # minusmultiply_const_2() 2
    function minusmultiply_const_2() public pure returns (uint) {
        return 8 - 2 * 3;
    }

    // # minusmultiply_const_11() 11
    function minusmultiply_const_11() public pure returns (uint) {
        return 20 - (3 * 3);
    }

    // # minusmultiply_const_18() 18
    function minusmultiply_const_18() public pure returns (uint) {
        return (8 - 2) * 3;
    }

    // # minusmultiply_const_0() 0
    function minusmultiply_const_0() public pure returns (uint) {
        return (8 - 8) * 3;
    }

    // # minusmultiply_params_with_const(6) 0
    // # minusmultiply_params_with_const(7) 1
    // # minusmultiply_params_with_const(0) !panic
    function minusmultiply_params_with_const(uint a) public pure returns (uint) {
        uint b = 2;
        uint c = 3;
        return a - b * c;
    }
}
