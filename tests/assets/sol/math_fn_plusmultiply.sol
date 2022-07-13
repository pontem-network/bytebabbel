// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library MathPlusMultiplyFn {
    // # plusmultiply_const_1() 7
    function plusmultiply_const_1() public pure returns (uint) {
        return 1 + 2 * 3;
    }

    // # plusmultiply_const_2() 7
    function plusmultiply_const_2() public pure returns (uint) {
        uint a = 1;
        uint b = 2;
        uint c = 3;
        return a + b * c;
    }

    // # plusmultiply_const_3() 7
    function plusmultiply_const_3() public pure returns (uint) {
        uint a = 1;
        uint b = 2;
        uint c = 3;
        return a + (b * c);
    }

    // # plusmultiply_const_9() 9
    function plusmultiply_const_9() public pure returns (uint) {
        uint a = 1;
        uint b = 2;
        uint c = 3;
        return (a + b) * c;
    }
}
