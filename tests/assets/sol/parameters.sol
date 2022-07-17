// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

contract Parameters {

//    // # plus (2, 3) 5_u128
//    // # plus (1, 18446744073709551615) 18446744073709551616_u128
//    // # plus (18446744073709551615, 1) 18446744073709551616_u128
//    // # plus (1, 18446744073709551615) 18446744073709551616_u128
//    // # plus (340282366920938463463374607431768211455,0) 340282366920938463463374607431768211455_u128
//    // # plus (0,340282366920938463463374607431768211455) 340282366920938463463374607431768211455_u128
//    // # plus (340282366920938463463374607431768211455,1) !panic
//    // # plus (true,1) !panic
//    // # plus (1, true) !panic
//    // # plus (1) !panic
//    function plus(uint a, uint b) public pure returns (uint) {
//        return a + b;
//    }
//
//    // # a_or_b (1, 2, true) 1
//    // # a_or_b (1, 2, false) 2
//    // # a_or_b (1, 2, 1) !panic
//    function a_or_b(uint a, uint b, bool is_a) public pure returns (uint) {
//        if (is_a) {
//            return a;
//        } else {
//            return b;
//        }
//    }
//
//    // # is_zero (1) false
//    // # is_zero (0) true
//    function is_zero(uint a) public pure returns (bool) {
//        return a == 0;
//    }

    // # minusmultiply_params_with_const(6) 12
    function minusmultiply_params_with_const_2(uint a) public pure returns (uint) {
        uint b = 2;
        uint c = 3;
        return (a - b) * c;
    }
}
