// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

contract Simple {

    // # f_bytes ()
    function f_bytes() public pure {
        bytes1 a = bytes1(uint8(0));
        bytes2 b = bytes2(uint16(0));
        assert(a[0] == b[0]);

    }

    // # f_bytes_dyn ()
    function f_bytes_dyn() public pure {
        bytes memory b = new bytes(200);
        b[0] = bytes1(uint8(0));
    }

    // # ignore_f_bytes_dyn_hex ()
    function f_bytes_dyn_hex() public pure {
        bytes memory b = hex"0000";
        assert(b[0] == bytes1(uint8(0)));
    }


}
