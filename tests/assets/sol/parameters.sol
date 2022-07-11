// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

contract Parameters {
    function plus(uint a, uint b) public pure returns(uint) {
       return a + b;
    }

    function a_or_b(uint a, uint b, bool is_a) public pure returns(uint) {
        if (is_a) {
            return a;
        } else {
            return b;
        }
    }

    function is_zero(uint a) public pure returns(bool) {
        return a == 0;
    }
}
