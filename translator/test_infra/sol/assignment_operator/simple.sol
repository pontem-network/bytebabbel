// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

contract Simple {

    // # plus (20) 30
    function plus(uint a) public pure returns (uint) {
        a+=10;
        return a;
    }

    // # minus (20) 10
    function minus(uint a) public pure returns (uint) {
        a-=10;
        return a;
    }

    // # divede (200) 20
    function divede(uint a) public pure returns (uint) {
        a/=10;
        return a;
    }

    // # multiply (10) 100
    function multiply(uint a) public pure returns (uint) {
        a*=10;
        return a;
    }

    // # mod (12) 2
    function mod(uint a) public pure returns (uint) {
        a%=10;
        return a;
    }
}
