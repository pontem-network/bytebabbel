// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

contract Simple {

    // # plus (20) 30
    function plus(uint a) public pure returns (uint) {
        a += 10;
        return a;
    }

    // # minus (20) 10
    function minus(uint a) public pure returns (uint) {
        a -= 10;
        return a;
    }

    // # divede (200) 20
    function divede(uint a) public pure returns (uint) {
        a /= 10;
        return a;
    }

    // # multiply (10) 100
    function multiply(uint a) public pure returns (uint) {
        a *= 10;
        return a;
    }

    // # and_uint (20) 20
    // # and_uint (10) 0
    function and_uint(uint a) public pure returns (uint) {
        a &= 20;
        return a;
    }
}
