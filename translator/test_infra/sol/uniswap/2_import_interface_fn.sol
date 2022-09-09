// SPDX-License-Identifier: Apache-2.0

pragma solidity >=0.8.10;

import './i_import_interface.sol';

contract Fn is iFn {

    // # check () true
    function check() external pure returns (bool){
        return true;
    }
}
