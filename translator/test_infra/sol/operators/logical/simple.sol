// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.10;

contract LogicalOperators {
    // # greater_uint (6) true
    // # greater_uint (5) false
    // # greater_uint (4) false
    function greater_uint(uint a) public pure returns (bool) {
        return a > 5;
    }

    // # eq_greater_uint (6) true
    // # eq_greater_uint (5) true
    // # eq_greater_uint (4) false
    function eq_greater_uint(uint a) public pure returns (bool) {
        return a >= 5;
    }

    // # less_uint (6) false
    // # less_uint (5) false
    // # less_uint (4) true
    function less_uint(uint a) public pure returns (bool) {
        return a < 5;
    }

    // # eq_less_uint (6) false
    // # eq_less_uint (5) true
    // # eq_less_uint (4) true
    function eq_less_uint(uint a) public pure returns (bool) {
        return a <= 5;
    }

    // # equals_uint (6) false
    // # equals_uint (5) true
    // # equals_uint (4) false
    function equals_uint(uint a) public pure returns (bool) {
        return a == 5;
    }

    // # equals_bool (true) true
    // # equals_bool (false) false
    function equals_bool(bool a) public pure returns (bool) {
        return a == true;
    }

    // # not_equals_uint (6) true
    // # not_equals_uint (5) false
    // # not_equals_uint (4) true
    function not_equals_uint(uint a) public pure returns (bool) {
        return a != 5;
    }

    // # not_equals_bool (true) false
    // # not_equals_bool (false) true
    function not_equals_bool(bool a) public pure returns (bool) {
        return a != true;
    }


    // # and_bool (true, true) true
    // # and_bool (true, false) false
    // # and_bool (false, false) false
    function and_bool(bool a, bool b) public pure returns (bool) {
        return a && b;
    }

    // # or_bool (true, true) true
    // # or_bool (true, false) true
    // # or_bool (false, false) false
    function or_bool(bool a, bool b) public pure returns (bool) {
        return a || b;
    }

    // # not_bool (true) false
    // # not_bool (false) true
    function not_bool(bool a) public pure returns (bool) {
        return !a;
    }

}
