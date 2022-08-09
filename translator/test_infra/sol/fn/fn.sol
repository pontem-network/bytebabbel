// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library Fn {
    // # fn_const_return_uint () 1
    function fn_const_return_uint() public pure returns (uint){
        return 1;
    }

    // # fn_const_return_bool () true
    function fn_const_return_bool() public pure returns (bool){
        return true;
    }

    function one() public pure returns (uint){
        return 1;
    }

    function two() public pure returns (uint){
        return 2;
    }

    // # one_two () 3
    function one_two() public pure returns (uint){
        return one() + two();
    }

    // # one_two_is_a (true) 1
    // # one_two_is_a (false) 2
    function one_two_is_a(bool a) public pure returns (uint){
        if (a) {
            return one();
        } else {
            return two();
        }
    }

    // # fn_const()
    function fn_const() public pure{ }

    // # test_3 (0, 1, 2) 1
    function test_3(uint a, uint b, uint c) public pure returns (uint){
        bool stop = a < 1;
        if (stop) {
            return b;
        }

        return b + c;
    }

    // # test_1 (0, 1, 2) 1
    // # test_1 (323232323, 1, 2) 2
    function test_1(uint a, uint b, uint c) public pure returns (uint){
        if (a == 0) {
            return b;
        } else {
            return c;
        }
    }
}
