// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity >=0.8.10;

import "./l_SafeMath.sol";

contract TestMapping {
    using SafeMath for uint;
    uint public totalSupply;

    mapping(address => uint) public balanceOf;

    event Transfer(address indexed from, address indexed to, uint value);

    // # mint(0x1,10) 10
    function mint(address to, uint value) public returns(uint){
        totalSupply = totalSupply.add(value);
        balanceOf[to] = balanceOf[to].add(value);
        emit Transfer(address(0), to, value);

        return balanceOf[to];
    }
}
