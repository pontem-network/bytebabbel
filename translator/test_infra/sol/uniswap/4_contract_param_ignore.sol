// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity >=0.8.10;

contract TestParam {
    uint public totalSupply;

    // # get_set(10) 10
    function get_set(uint value) public returns(uint){
        totalSupply = value;

        return totalSupply;
    }
}
