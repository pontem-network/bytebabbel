// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library Byte {
    // # byte_tests (30, 1024)
    // # byte_tests (30, 27818238)
    // # byte_tests (26, 1238712387123)
    // # byte_tests (26, 12387123871231238728)
    // # byte_tests (20, 12387123871231238728283172387)
    function byte_test(uint256 a, uint256 b) public pure returns (uint256 r) {
        assembly {
            r := byte(a, b)
        }
    }
}
