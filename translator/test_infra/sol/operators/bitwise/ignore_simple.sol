// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

contract Simple {
    // # not_uint128 (20) 65515
    function not_uint(uint128 a) public pure returns (uint128) {
        return ~a;
    }
}
