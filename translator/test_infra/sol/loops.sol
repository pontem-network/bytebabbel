// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library  Loop {
//    function for_loop(uint i) public pure returns (uint) {
//        uint val = 0;
//        for (uint j = 0; j < i; j++) {
//            val += j * i;
//        }
//        return val;
//    }
    function for_loop(uint i) public pure returns (uint) {
       uint val = 1000;
        if (i != 0) {
           val += i;
       } else {
           val += i;
       }
        return val;
    }
}
