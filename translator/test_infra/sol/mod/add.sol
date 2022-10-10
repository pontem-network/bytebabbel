// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library AddMod {
    // # add_mod_u256 (100, 5, 2000) 105
    // # add_mod_u256 (115792089237316195423570985008687907853269984665640564039457584007913129639934, 115792089237316195423570985008687907853269984665640564039457584007913129639932, 4) 2
    // # add_mod_u256 (42971, 8723, 793) 149
    function add_mod_u256(uint256 a, uint256 b, uint256 mod) public pure returns (uint256) {
        return addmod(a, b, mod);
    }

    // # add_mod_u256_max () 202
    function add_mod_u256_max() public pure returns (uint256) {
        uint256 MAX_INT = 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;

        return addmod(MAX_INT - 2, MAX_INT - 2, MAX_INT - 103);
    }

}
