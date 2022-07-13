// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library MathPlusFn {
    // # plus_const_3() 3
    function plus_const_3() public pure returns (uint) {
        uint a = 1;
        uint b = 2;
        return a + b;
    }

    // # plus_const_4() 4
    function plus_const_4() public pure returns (uint) {
        uint a = 1;
        uint b = 3;
        uint c = a + b;
        return c;
    }

    // # plus_const_15() 15
    function plus_const_15() public pure returns (uint) {
        uint a = 1 + 3 + 4;
        uint b = 2;
        return a + b + 5;
    }

    // # plus_const_30() 30
    function plus_const_30() public pure returns (uint) {
        uint a = 1;
        uint b = 2;
        uint c = (a + b) + (3 + 4 + 5);
        c = c + 5;
        uint d = 10;
        return d + c;
    }

    // # plus_params(1,2,3) 6
    // # plus_params(18446744073709551615,0,0) 18446744073709551615
    // # plus_params(18446744073709551615,18446744073709551615,18446744073709551615) 55340232221128654845
    // # plus_params(340282366920938463463374607431768211455,0,0) 340282366920938463463374607431768211455
    // # plus_params(340282366920938463463374607431768211455,1,0) !panic
    function plus_params(uint a, uint b, uint c) public pure returns (uint) {
        return a + b + c;
    }


}
