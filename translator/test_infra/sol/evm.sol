// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.4.16 <0.9.0;

contract Demo {
    bool private value;

    function without_params_bool() public pure returns (bool){
        return true;
    }

    function param_bool(bool val1, bool val2) public pure returns (bool){
        return val1 && val2;
    }

    function with_uint(uint128 a) public pure returns (uint128){
        return a * a;
    }

    function max_num_tuple(int8 a1, int16 a2, uint32 b1, uint64 b2) public pure returns (int, uint){
        int a3;
        if (a1 > a2) {
            a3 = a1;
        } else {
            a3 = a2;
        }

        uint128 b3;
        if (b1 > b2) {
            b3 = b1;
        } else {
            b3 = b2;
        }
        return (a3, b3);
    }

    function array_bool_3() public pure returns (bool[3] memory){
        bool[3] memory val = [true, true, false];
        return val;
    }

    function array_bool_dyn() public pure returns (bool[] memory){
        bool[] memory val = new bool[](4);
        val[0] = true;
        val[1] = false;
        val[2] = false;
        val[3] = true;
        return val;
    }

    function array_bool_dyn2() public pure returns (bool[][] memory){
        bool[][] memory val = new bool[][](2);
        val[0] = new bool[](2);
        val[0][0] = false;
        val[0][1] = true;
        val[1] = new bool[](1);
        val[1][0] = true;

        return val;
    }

    function array_bool_dyn3() public pure returns (bool[] memory,bool[][] memory){
        return (array_bool_dyn(), array_bool_dyn2());
    }

    function byte_tuple() public pure returns (bytes memory, bytes3, bytes1[2] memory, bytes2[] memory){
        bytes memory bs_dyn = new bytes(2);
        bs_dyn = "01";
        bytes3 bs3 = "123";
        bytes1[2] memory bs1 = [bytes1("0"),"1"];
        bytes2[] memory bs2 = new bytes2[](3);
        bs2[0] = "00";
        bs2[1] = "01";
        bs2[2] = "02";

        return (bs_dyn, bs3, bs1, bs2);
    }
}


