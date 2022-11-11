// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library Fn {
    // # fn_const_return_uint128 ()
    function fn_const_return_uint() public pure returns (uint128){
        return 1;
    }

    // # fn_const_return_bool ()
    function fn_const_return_bool() public pure returns (bool){
        return true;
    }

    function one() public pure returns (uint128){
        return 1;
    }

    function two() public pure returns (uint128){
        return 2;
    }

    // # one_two ()
    function one_two() public pure returns (uint128){
        return one() + two();
    }

    // # one_two_is_a (true)
    // # one_two_is_a (false)
    function one_two_is_a(bool a) public pure returns (uint128){
        if (a) {
            return one();
        } else {
            return two();
        }
    }

    // # fn_const()
    function fn_const() public pure {}

    // # test_3 (0, 1, 2)
    function test_3(uint128 a, uint128 b, uint128 c) public pure returns (uint128){
        if (a < 1) {
            return b;
        }

        return b + c;
    }

    // # test_1 (0, 1, 2)
    // # test_1 (323232323, 1, 2)
    function test_1(uint128 a, uint128 b, uint128 c) public pure returns (uint128){
        if (a == 0) {
            return b;
        } else {
            return c;
        }
    }
}
