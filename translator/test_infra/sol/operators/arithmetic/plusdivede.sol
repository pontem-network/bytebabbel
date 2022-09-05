// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library PlusDevide {
    // # plusdevide_const_3() 3
    function plusdevide_const_3() public pure returns (uint128) {
        return 1 + 4 / 2;
    }

    // # plusdevide_const_5() 5
    function plusdevide_const_5() public pure returns (uint128) {
        return 1 + (4 / 1);
    }

    // # plusdevide_const_4() 4
    function plusdevide_const_4() public pure returns (uint128) {
        return (4 + 4) / 2;
    }
}
