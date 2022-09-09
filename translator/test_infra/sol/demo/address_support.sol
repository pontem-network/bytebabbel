// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

contract AddressSupport {
   address owner;

    constructor() {
        owner = msg.sender;
    }

   function is_owner() public returns (bool) {
       return owner == msg.sender;
   }
}
