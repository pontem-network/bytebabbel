// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

contract Simple {

    // # f_array()
    function f_array() public pure {
        uint[3] memory a = [uint(1), 2, 3];
        uint[3] memory b = [uint(1), 3, 2];
        assert(a[2] == b[1]);
    }

    //  # f_array_t()
    function f_array_t() public pure {
        assert((255 + [1, 2, 3][0]) == 256);
    }
}
