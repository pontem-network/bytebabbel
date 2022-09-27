pragma solidity ^0.8.0;

contract Strings {
    string state = "Hello, World!";

    function const_str() public pure returns (string memory) {
        return "hello";
    }

    function get_state() public view returns (string memory) {
        return state;
    }

//    function set_state(string memory s) public {
//        state = s;
//    }
}
/*
 if (!var6) {
        label_00FF:
            return var1;
        } else if (0x1f < var6) {
            var temp3 = var4;
            var temp4 = temp3 + var6;
            var4 = temp4;
            memory[0x00:0x20] = var5;
            var temp5 = keccak256(memory[0x00:0x20]);
            memory[temp3:temp3 + 0x20] = storage[temp5];
            var5 = temp5 + 0x01;
            var6 = temp3 + 0x20;

            if (var4 <= var6) { goto label_00F6; }

        label_00E2:
            var temp6 = var5;
            var temp7 = var6;
            memory[temp7:temp7 + 0x20] = storage[temp6];
            var5 = temp6 + 0x01;
            var6 = temp7 + 0x20;

            if (var4 > var6) { goto label_00E2; }

        label_00F6:
            var temp8 = var4;
            var temp9 = temp8 + (var6 - temp8 & 0x1f);
            var6 = temp8;
            var4 = temp9;
            goto label_00FF;
        } else {
            var temp10 = var4;
            memory[temp10:temp10 + 0x20] = storage[var5] / 0x0100 * 0x0100;
            var4 = temp10 + 0x20;
            var6 = var6;
            goto label_00FF;
        }
*/