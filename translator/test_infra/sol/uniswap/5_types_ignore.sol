// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity >=0.8.10;

contract TestTypes {
    uint128 public v_uint128;
    uint8 public v_uint8;
    string public v_string;
    bytes32 public v_bytes32;
    address public v_address;

    // # get_set_uint(1) 1
    function get_set_uint(uint128 value) public returns (uint128){
        v_uint128 = value;
        return v_uint128;
    }

    // # get_set_uint8(1) 1
    function get_set_uint(uint8 value) public returns (uint8){
        v_uint8 = value;
        return v_uint8;
    }

    // # get_set_string("demo") "demo"
    function get_set_string(string memory value) public returns (string memory){
        v_string = value;
        return v_string;
    }

    // # get_set_bytes32("demo") "demo"
    function get_set_bytes32(bytes32 value) public returns (bytes32){
        v_bytes32 = value;
        return v_bytes32;
    }

    // # get_set_address(0x1) 0x1
    function get_set_address(address value) public returns (address){
        v_address = value;
        return v_address;
    }
}
