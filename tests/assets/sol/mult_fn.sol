// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library MultFn {
    function multiplication_6() public pure returns (uint) {
        return 2 * 3;
    }

    function multiplication_18() public pure returns (uint) {
        uint a = 2;
        uint b = 3;
        return a * b * 3;
    }


    function multiplication_36() public pure returns (uint) {
        uint a = 2;
        uint b = 3 * 2;
        return a * b * 3;
    }

    function multiplication_3888() public pure returns (uint) {
        uint a = 2;
        uint b = 3 * 2; // 6
        uint c = (a * b) * 3; // 36
        a = c * b; // 216
        return a * b * 3; // 3888
    }
}
