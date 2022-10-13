// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library Info {
    // # sender_address ()
    function sender_address() public view returns (address){
        return msg.sender;
    }

    // #balance# sender_balance ()
    function sender_balance() public view returns (uint){
        return msg.sender.balance;
    }

    // #balance# x42_balance ()
    function x42_balance() public view returns (uint){
        address user = 0x0000000000000000000000000000000000000042;
        return user.balance;
    }

    // # non_existent_address ()
    function non_existent_address() public view returns (uint){
        address user = 0x0000000000000000000000000000000000000066;
        return user.balance;
    }

    // # gas_price ()
    function gas_price() public view returns (uint){
        uint256 gasPrice;
        assembly {
            gasPrice := gasprice()
        }
        return gasPrice;
    }

    // # gas_limit ()
    function gas_limit() public view returns (uint){
        uint256 limit;
        assembly {
            limit := gaslimit()
        }
        return limit;
    }

    // #block#  ignore_block_height()
    function block_height() public view returns (uint){
        return block.number;
    }

    // #block# epoch_interval_secs()
    function epoch_interval_secs() public view returns (uint){
        return block.timestamp;
    }
}

