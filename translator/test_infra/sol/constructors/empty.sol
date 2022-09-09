// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

contract empty {
    uint val;

    constructor() {
        val = 42;
        return;
    }

    function get_val() public view returns (uint256) {
        return val;
    }
}
