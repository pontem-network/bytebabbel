// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library MinusDevide {

    // # minusdevide_const_4() 4
    function minusdevide_const_4() public pure returns (uint) {
        return 8 - 8 / 2;
    }

    // # minusdevide_const_19() 19
    function minusdevide_const_19() public pure returns (uint) {
        return 20 - (3 / 3);
    }

    // # minusdevide_const_2() 2
    function minusdevide_const_2() public pure returns (uint) {
        return (8 - 2) / 3;
    }

    // # minusdevide_const_0() 0
    function minusdevide_const_0() public pure returns (uint) {
        return (8 - 8) / 3;
    }

    // # minusdevide_params(6) 3
    // # minusdevide_params(3) 0
    // # minusdevide_params(2) !panic
    function minusdevide_params(uint a) public pure returns (uint) {
        return a - 6 / 2;
    }

    // # minusdevide_params_2(6) 3
    // # minusdevide_params_2(3) 0
    // # minusdevide_params_2(2) !panic
    function minusdevide_params_2(uint a) public pure returns (uint) {
        uint b = 6;
        uint c = 2;
        return a - b / c;
    }
}
