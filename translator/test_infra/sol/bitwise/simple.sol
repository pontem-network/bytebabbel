// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

contract Simple {

    // # and_uint (20, 10) 0
    // # and_uint (20, 20) 20
    function and_uint(uint a, uint b) public pure returns (uint) {
        return a & b;
    }
}
