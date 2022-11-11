// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

contract TypeString {

    // # f_string()
    function f_string() public pure {
        string memory a = "demo";
        string memory b = "demo2";
        assert(bytes(a)[0] != bytes(b)[0]);
    }

    // # f_return_string ()
    function f_return_string() public pure returns (string memory) {
        string memory s = "demo2";
        return s;
    }

    // # ignore_f_input_output ("demo")
    function f_input_output(string memory s) public pure returns (string memory) {
        return s;
    }

}
