// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

contract TypeInt {
    // # f_int (100) 100
    function f_int(int a) public pure returns (int) {
        return a;
    }

    // # f_int8 (100) 100
    // ?
    // # f_int8 (100u8) 100u8
    function f_int8(int8 a) public pure returns (int8) {
        return a;
    }

    // # f_int16 (100) 100
    function f_int16(int16 a) public pure returns (int16) {
        return a;
    }

    // # f_int32 (100) 100
    function f_int32(int32 a) public pure returns (int32) {
        return a;
    }

    // # f_int64 (100) 100
    function f_int64(int64 a) public pure returns (int64) {
        return a;
    }

    // # f_int128 (100) 100
    function f_int128(int128 a) public pure returns (int128) {
        return a;
    }

    // # f_int256 (100) 100
    function f_int256(int256 a) public pure returns (int256) {
        return a;
    }
}