// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library PlusMinus {
    // # plusminus_const_106()
    function plusminus_const_106() public pure returns (uint128) {
        uint128 a = 128 - 12;
        uint128 b = 28 + 12;
        uint128 c = 30;
        return a - b + c;
    }

    // # plusminus_const_40()
    function plusminus_const_40() public pure returns (uint128) {
        uint128 a = 10;
        uint128 b = 20;
        uint128 c = 50 - 20;
        a = a + (c - b);
        return a + a;
    }

    // # plusminus_params(0,0,0)
    // # plusminus_params(1,2,3)
    // # plusminus_params(1,2,4)
    // # ignore_plusminus_params(340282366920938463463374607431768211454,1,340282366920938463463374607431768211455)
    // # ignore_plusminus_params(340282366920938463463374607431768211455,0,340282366920938463463374607431768211455)
    // # ignore_plusminus_params(0,340282366920938463463374607431768211455,0)
    // # ignore_plusminus_params(340282366920938463463374607431768211455,340282366920938463463374607431768211455,340282366920938463463374607431768211455)
    function plusminus_params(uint128 a, uint128 b, uint128 c) public pure returns (uint128) {
        return a + b - c;
    }

    // # plusminus_params_2(1,2,3)
    // # plusminus_params_2(1,2,4)
    // # ignore_plusminus_params_2(340282366920938463463374607431768211454,1,340282366920938463463374607431768211455)
    // # ignore_plusminus_params_2(340282366920938463463374607431768211455,0,340282366920938463463374607431768211455)
    // # ignore_plusminus_params_2(0,340282366920938463463374607431768211455,0)
    // # ignore_plusminus_params_2(340282366920938463463374607431768211455,340282366920938463463374607431768211455,340282366920938463463374607431768211455)
    function plusminus_params_2(uint128 a, uint128 b, uint128 c) public pure returns (uint128) {
        return (a + b) - c;
    }

    // # plusminus_params_3(0,0,0)
    // # plusminus_params_3(3,2,1)
    // # plusminus_params_3(3,1,2)
    // # ignore_plusminus_params_3(340282366920938463463374607431768211455,340282366920938463463374607431768211455,340282366920938463463374607431768211455)
    // # ignore_plusminus_params_3(340282366920938463463374607431768211455,340282366920938463463374607431768211455,0)
    // # ignore_plusminus_params_3(340282366920938463463374607431768211455,0,340282366920938463463374607431768211455)
    function plusminus_params_3(uint128 a, uint128 b, uint128 c) public pure returns (uint128) {
        return a + (b - c);
    }


    // # plusminus_params_4(0,0,0)
    // # plusminus_params_4(6,1,2)
    // # ignore_plusminus_params_4(340282366920938463463374607431768211455,340282366920938463463374607431768211455,0)
    // # ignore_plusminus_params_4(340282366920938463463374607431768211455,340282366920938463463374607431768211455,340282366920938463463374607431768211455)
    function plusminus_params_4(uint128 a, uint128 b, uint128 c) public pure returns (uint128) {
        return a - b + c;
    }

    // # plusminus_params_5(0,0,0)
    // # plusminus_params_5(6,1,2)
    // # ignore_plusminus_params_5(340282366920938463463374607431768211455,340282366920938463463374607431768211455,0)
    // # ignore_plusminus_params_5(340282366920938463463374607431768211455,340282366920938463463374607431768211455,340282366920938463463374607431768211455)
    // # ignore_plusminus_params_5(340282366920938463463374607431768211455,0,340282366920938463463374607431768211455)
    function plusminus_params_5(uint128 a, uint128 b, uint128 c) public pure returns (uint128) {
        return (a - b) + c;
    }

    // # plusminus_params_6(0,0,0)
    // # plusminus_params_6(6,1,2)
    // # ignore_plusminus_params_6(340282366920938463463374607431768211455,0,340282366920938463463374607431768211455)
    // # ignore_plusminus_params_6(340282366920938463463374607431768211455,340282366920938463463374607431768211455,340282366920938463463374607431768211455)
    function plusminus_params_6(uint128 a, uint128 b, uint128 c) public pure returns (uint128) {
        return a - (b + c);
    }

}
