// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library WhileLoop {
    // # sum (2,10) 20
    function sum(uint inc, uint limit) public pure returns (uint) {
        uint result = 0;
        for(uint i=0; i<limit; i++) {
            result += inc;
        }
        return result;
    }

}
