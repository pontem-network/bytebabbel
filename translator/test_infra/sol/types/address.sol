// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

contract Simple {

    // # f_address()
    function f_address() public pure {
        address a = 0x111122223333444455556666777788889999aAaa;
        address b = 0x777788889999AaAAbBbbCcccddDdeeeEfFFfCcCc;
        assert(a!=b);
    }
}
