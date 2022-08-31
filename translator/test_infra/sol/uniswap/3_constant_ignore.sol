// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity >=0.8.10;

contract TestConstant {
    string public constant name = 'Uniswap V2';
    uint8 public constant decimals = 18;
    bytes32 public constant PERMIT_TYPEHASH = 0x6e71edae12b1b97f4d1f60370fef10105fa2faae0126114a169c64845d6126c9;


    // # get_name() "Uniswap V2"
    function get_name() public pure returns (string memory){
        return name;
    }

    // # get_decimals() 18
    function get_decimals() public pure returns (uint8){
        return decimals;
    }

    // # get_permit_typehash() 0x6e71edae12b1b97f4d1f60370fef10105fa2faae0126114a169c64845d6126c9
    function get_permit_typehash() public pure returns (bytes32){
        return PERMIT_TYPEHASH;
    }
}
