// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

contract Simple {
    // # not_uint (20) 65515
    function not_uint(uint a) public pure returns (uint) {
        return ~a;
    }
}
