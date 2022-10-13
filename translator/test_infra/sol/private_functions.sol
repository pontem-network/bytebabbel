pragma solidity ^0.8.0;

contract private_functions {
    function test_1(bool a, bool b, bool c) public pure returns (uint128){
        if (a) {
            if (b) {
                if (c) {
                    return on_first_case();
                }
                return on_second_case();
            } else {
                return on_third_case();
            }
        }

        return default_case();
    }

    function on_first_case() private pure returns (uint128) {
        uint128 a = 10;
        uint128 b = 2;
        uint128 c = 3;
        return a + b + c;
    }

    function on_second_case() private pure returns (uint128) {
        uint128 a = 100;
        uint128 b = 20;
        uint128 c = 30;
        return a + b + c;
    }

    function on_third_case() private pure returns (uint128) {
        uint128 a = 1000;
        uint128 b = 200;
        uint128 c = 300;
        return a + b + c;
    }

    function default_case() private pure returns (uint128) {
        uint128 a = 10000;
        uint128 b = 2000;
        uint128 c = 3000;
        return a + b + c;
    }
}
