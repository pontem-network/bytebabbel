// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library Simple {
    // # mod(9,5) 4
    function mod(uint a, uint b) public pure returns (uint) {
        return a % b;
    }

    // # inc(2) 3
    function inc(uint a) public pure returns (uint) {
        return ++a;
    }

}
