// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

type DT is uint128;

contract Simple {
    // # test_1(2) 2
    function test_1(DT a) public pure returns (DT){
        return a;
    }
}
