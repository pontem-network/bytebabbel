// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library IfFn {
    // # test_1 (true, true, true) 3
    // # test_1 (true, false, true) 1
    // # test_1 (true, false, true) 1
    // # test_1 (true, true, false) 2
    // # test_1 (false, false, false) 0
    function test_1(bool a, bool b, bool c) public pure returns (uint128){
        if (a) {
            if(b){
                if(c){
                    return 3;
                }
                return 2;
            }else{
                return 1;
            }
        }

        return 0;
    }

    // # test_2 (true, 1, 2) 1
    // # test_2 (false, 1, 2) 3
    function test_2(bool a, uint128 b, uint128 c) public pure returns (uint128){
        if (a) {
            return b;
        }

        return b + c;
    }

    // # test_3 (1, 2) 2
    // # test_3 (256, 0) 256
    function test_3(uint128 a, uint128 b) public pure returns (uint128){
        return  (a > b ? a : b);
    }
}
