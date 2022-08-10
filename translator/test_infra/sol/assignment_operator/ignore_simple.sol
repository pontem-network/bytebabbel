// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

contract Simple {
    // # mod (12) 2
    function mod(uint a) public pure returns (uint) {
        a%=10;
        return a;
    }
}
