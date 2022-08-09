// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library Recursion {
    // # recursion_sum_1(0,256) 256
    // # recursion_sum_1(256,256) 512
    function recursion_sum_1(uint value, uint limit)  public pure returns (uint){
        if (limit == 0) {
            return value;
        }
        uint limit_2 = limit - 1;
        uint value_2 = value + 1;
        return recursion_sum_1(value_2, limit_2);
    }

    // # recursion_sum_1(0,256) 256
    // # recursion_sum_1(256,256) 512
    function recursion_sum_2(uint value, uint limit)  public pure returns (uint){
        if (limit == 0) {
            return recursion_sum_2(value + 1, limit - 1);
        }
        return value;
    }
}
