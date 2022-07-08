// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library ConstFn {
    function const_fn_10() public pure returns (uint) {
        return 10;
    }

    function const_fn_426574676453456() public pure returns (uint) {
        return 426574676453456;
    }

    function const_fn_true() public pure returns (bool) {
        return true;
    }

    function const_fn_90_plus_54() public pure returns (uint) {
        uint a = 90;
        uint b = 54;
        return a + b;
    }

    function const_fn_true_1() public pure returns (bool) {
        bool a = false;
        bool b = true;
        return a == b;
    }
}
