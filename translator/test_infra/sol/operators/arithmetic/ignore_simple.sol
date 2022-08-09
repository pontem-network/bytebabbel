// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library Simple {
    // # mod(9,5) 4
    function mod(uint a, uint b) public pure returns (uint) {
        return a % b;
    }

    // # exp(2,2) 4
    // # exp(2,3) 8
    // # exp(3,3) 27
    function exp(uint a, uint b) public pure returns (uint) {
        return a ** b;
    }

    // # inc_pre(2) 11
    function inc_pre(uint a) public pure returns (uint) {
        uint c = 8;
        uint b = ++c + a;
        assert(c == 9);
        return b;
    }
}
