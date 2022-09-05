// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library Plus {
    // # plus_const_3()
    function plus_const_3() public pure returns (uint128) {
        uint128 a = 1;
        uint128 b = 2;
        return a + b;
    }

    // # plus_const_4()
    function plus_const_4() public pure returns (uint128) {
        uint128 a = 1;
        uint128 b = 3;
        uint128 c = a + b;
        return c;
    }

    // # plus_const_15()
    function plus_const_15() public pure returns (uint128) {
        uint128 a = 1 + 3 + 4;
        uint128 b = 2;
        return a + b + 5;
    }

    // # plus_const_30()
    function plus_const_30() public pure returns (uint128) {
        uint128 a = 1;
        uint128 b = 2;
        uint128 c = (a + b) + (3 + 4 + 5);
        c = c + 5;
        uint128 d = 10;
        return d + c;
    }
}
