// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library MinusDevide {

    // # minusdevide_const_4()
    function minusdevide_const_4() public pure returns (uint128) {
        return 8 - 8 / 2;
    }

    // # minusdevide_const_19()
    function minusdevide_const_19() public pure returns (uint128) {
        return 20 - (3 / 3);
    }

    // # minusdevide_const_2()
    function minusdevide_const_2() public pure returns (uint128) {
        return (8 - 2) / 3;
    }

    // # minusdevide_const_0()
    function minusdevide_const_0() public pure returns (uint128) {
        return (8 - 8) / 3;
    }

    // # minusdevide_params(6)
    // # minusdevide_params(3)
    // # minusdevide_params(2)
    function minusdevide_params(uint128 a) public pure returns (uint128) {
        return a - 6 / 2;
    }

    // # minusdevide_params_2(6)
    // # minusdevide_params_2(3)
    // # minusdevide_params_2(2)
    function minusdevide_params_2(uint128 a) public pure returns (uint128) {
        uint128 b = 6;
        uint128 c = 2;
        return a - b / c;
    }
}
