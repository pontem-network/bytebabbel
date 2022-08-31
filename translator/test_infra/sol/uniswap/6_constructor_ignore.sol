// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity >=0.8.10;

contract TestConst {
    uint uval;

    constructor() {
        uval = 0;
    }

    // # get() 0
    function get() public view returns (uint){
        return uval;
    }

    function set(uint a) public {
        uval = a;
    }

    // # get_set(10) 10
    function get_set(uint a) public returns (uint){
        uval = a;
        return uval;
    }
}
