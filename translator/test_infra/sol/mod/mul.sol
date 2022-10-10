// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library MulMod {
    // # mul_mod_u256 (2, 2, 5) 4
    // # mul_mod_u256 (115792089237316195423570985008687907853269984665640564039457584007913129639934, 115792089237316195423570985008687907853269984665640564039457584007913129639932, 27531917391) 2725202163
    // # mul_mod_u256 (301, 8261, 281) 273
    function mul_mod_u256(uint256 a, uint256 b, uint256 mod) public pure returns (uint256) {
        return mulmod(a, b, mod);
    }

    // # mul_mod_u256_max () 10201
    function mul_mod_u256_max() public pure returns (uint256) {
        uint256 MAX_INT = 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;

        return mulmod(MAX_INT - 2, MAX_INT - 2, MAX_INT - 103);
    }

}
