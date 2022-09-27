// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

library ForLoop {
//    // # sum (10, 10) 100
//    function sum(uint128 inc, uint128 limit) public pure returns (uint128, uint256) {
//        uint128 result = 0;
//        uint256 j = 10000000000000000 - 9;
//        for(uint128 i = 0; i < limit; i++) {
//            result += inc;
////            j -= 1;
//        }
//
//        return (result, j);
//    }

    function sum(uint128 inc, uint128 limit) public pure returns (uint128, uint256) {
        uint128 result = 0;
        while (limit > 0) {
            --limit;
            result += inc;
        }
        return (result, 10000000000000000 - 9);
    }


}

/*
loop {
    if (arg1 & 0xffffffffffffffffffffffffffffffff <= 0x00) {
        arg0 = 0x2386f26fc0fff7;
        r0 = var2;
        break r0, arg0;
    } else {
        var var4 = arg1;
        var var3 = 0x0096;
        var3 = func_01CD(var4);
        arg1 = var3;
        var3 = 0x00a4;
        var var5 = var2;
        var4 = arg0;
        var3 = func_01F6(var4, var5);
        var2 = var3;
        continue;
    }
}

 if (arg1 & 0xffffffffffffffffffffffffffffffff <= 0x00) {
        label_00AB:
            arg0 = 0x2386f26fc0fff7;
            r0 = var2;
            return r0, arg0;
        } else {
        label_008D:
            var var4 = arg1;
            var var3 = 0x0096;
            var3 = func_01CD(var4);
            arg1 = var3;
            var3 = 0x00a4;
            var var5 = var2;
            var4 = arg0;
            var3 = func_01F6(var4, var5);
            var2 = var3;

            if (arg1 & 0xffffffffffffffffffffffffffffffff <= 0x00) { goto label_00AB; }
            else { goto label_008D; }
        }
*/
