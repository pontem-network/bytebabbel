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

//    function for_loop(uint n) public view returns(uint) {
//        if (n <= 1) {
//            return n;
//        } else {
//            return for_loop(n - 1) + for_loop(n - 2);
//        }
//    }

//    function for_loop(uint n) public pure returns(uint) {
//        uint val = 0;
//        uint bff = 0;
//        for (uint i = 0; i < n; i++) {
//            for (uint j = 0; j < n; j++) {
//                val += i * j;
//                bff += i + j;
//            }
//        }
//
//        return val + bff;
//    }

    function for_loop(uint i) public pure returns (uint) {
        uint val = 1000;
        for (uint j = 0; j < 10; j++) {
            if (j / 2 == 0) {
                break;
            }
            val += j;
        }
        return val;
    }

//    function for_loop(uint i) public pure returns (uint) {
//       if (i /2 == 0) {
//           return i;
//       } else {
//           return i -1;
//       }
//    }

//    function for_loop(uint i) public pure returns (uint) {
//       uint val = 1000;
//        if (i != 0) {
//           val += i;
//       } else {
//           val += i;
//       }
//        return val;
//    }
}
