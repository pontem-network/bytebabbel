// SPDX-License-Identifier: Apache-2.0

pragma solidity >=0.8.10;

contract TestParam {
    uint128 public totalSupply;

    // # get_set(10) 10
    function get_set(uint128 value) public returns(uint128){
        totalSupply = value;

        return totalSupply;
    }
}
