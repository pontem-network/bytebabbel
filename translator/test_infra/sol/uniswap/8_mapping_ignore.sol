// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity >=0.8.10;

import "./l_SafeMath.sol";

contract TestMapping {
    mapping(address => uint128) public balanceOf;

    // # mint(0x1,10) 10
    function mint(address to, uint128 value) public returns(uint128){
        balanceOf[to] = balanceOf[to] + value;
        return balanceOf[to];
    }
}
