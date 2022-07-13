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
}
