// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library WhileLoop {
    // # sum (2,10) 20
    function sum(uint128 inc, uint128 limit) public pure returns (uint128) {
        uint128 result = 0;
        for(uint128 i=0; i<limit; i++) {
            result += inc;
        }
        return result;
    }

}
