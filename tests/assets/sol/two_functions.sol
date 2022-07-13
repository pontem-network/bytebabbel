// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

contract TwoFunctions {
    // # panic() !panic
    function panic() pure public {
        assert(false);
    }

    // @todo # do_nothing()
    // @todo # do_nothing(123) void !panic
    function do_nothing() public {}

    // @todo # boo(true)
    // @todo # boo(false)
    // @todo # boo(123) !panic
    function boo(bool s) public {
    }
}
