// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity >=0.8.10;

import "./l_SafeMath.sol";

contract TestMapping {
    mapping(address => uint) public balanceOf;

    // # mint(0x1,10) 10
    function mint(address to, uint value) public returns(uint){
        balanceOf[to] = balanceOf[to] + value;
        return balanceOf[to];
    }
}
