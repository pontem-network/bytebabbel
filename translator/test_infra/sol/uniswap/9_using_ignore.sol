// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity >=0.8.10;

import './l_SafeMath.sol';

contract Using {
    using SafeMath for uint128;

    // # add (1,2) 3
    function add(uint128 a, uint128 b) external pure returns (uint128){
        return a.add(b);
    }

    // # sub (3,2) 1
    function sub(uint128 a, uint128 b) external pure returns (uint128){
        return a.sub(b);
    }

    // # mul (3,2) 6
    function mul(uint128 a, uint128 b) external pure returns (uint128){
        return a.mul(b);
    }
}
