// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

contract WithIF {
    function minus(uint a, uint b) public pure returns (uint) {
        return a - b;
    }

    function plus(uint a, uint b) public pure returns (uint) {
        return a + b;
    }

    // # test1 (6,1) 5
    // # test1 (1,6) 5
    // # test1 (4,4) 8
    function test1(uint a, uint b) public pure returns (uint) {
        if (a > b) {
            return minus(a, b);
        } else if (a < b) {
            return minus(b, a);
        } else {
            return plus(a, b);
        }
    }

    // # test2 (true,true) 1
    // # test2 (true,false) 2
    // # test2 (false,true) 2
    // # test2 (false,false) 0
    function test2(bool a, bool b) public pure returns (uint) {
        if (a && b) {
            return 1;
        } else if (a || b) {
            return 2;
        } else {
            return 0;
        }
    }
}
