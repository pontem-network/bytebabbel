// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library ForLoop {
    // # for_loop (10, 10)
    // # for_loop (13, 1000)
    // # for_loop (49, 0)
    // # for_loop (42, 42)
    // # for_loop (0, 49)
    function for_loop(uint128 inc, uint128 limit) public pure returns (uint128, uint256) {
        uint128 result = 0;
        uint256 j = 10000000000000000;
        for(uint128 i = 0; i < limit; i++) {
            result += inc;
            j -= 1;
        }

        return (result, j);
    }

    // # for_static ()
    function for_static() public pure returns (uint128) {
        uint128 result = 0;
        for(uint128 i = 0; i < 10; i++) {
            result += 1;
        }
        return result;
    }
}
