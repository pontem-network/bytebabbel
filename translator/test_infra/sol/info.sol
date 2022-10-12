// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library Info {
    // # sender_address ()
    function sender_address() public view returns (address){
        return msg.sender;
    }

    // # sender_balance ()
    function sender_balance() public view returns (uint){
        return msg.sender.balance;
    }


    // # non_existent_address ()
    function non_existent_address() public view returns (uint){
        address user = 0x0000000000000000000000000000000000000066;
        return user.balance;
    }

    // # x42_balance ()
    function x42_balance() public view returns (uint){
        address user = 0x0000000000000000000000000000000000000042;
        return user.balance;
    }
}

