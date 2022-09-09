// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

contract with_data {
    uint256 val;

    constructor(uint init_val, bool cnd) {
        if (cnd) {
            val = init_val;
        } else {
            val = 42;
        }
    }

    function get_val() public view returns (uint256) {
        return val;
    }
}
