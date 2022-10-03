// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

contract Simple {

    // # f_string()
    function f_string() public pure {
        string memory a = "demo";
        string memory b = "demo2";
        assert(bytes(a)[0] != bytes(b)[0]);
    }
}
