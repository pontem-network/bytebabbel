// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

contract Simple {
    enum ActionChoices {GoLeft, GoRight, GoStraight, SitStill}
    ActionChoices constant defaultChoice = ActionChoices.GoStraight;

    // # f_default() 2
    function f_default() public pure returns (uint){
        return uint(defaultChoice);
    }

    // # f_min() 0
    function f_min() public pure returns (uint){
        return uint(type(ActionChoices).min);
    }

    // # f_max() 3
    function f_max() public pure returns (uint){
        return uint(type(ActionChoices).max);
    }
}
