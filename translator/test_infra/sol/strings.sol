pragma solidity ^0.8.0;

contract Strings {
    string state = "Hello, World!";

    function small_const_str() public pure returns (string memory) {
        return "hello";
    }

    function large_const_str() public pure returns (string memory) {
        return "This is the large string that we are testing. And it is bigger than 32 bytes.";
    }

//    function const_str_2() public pure returns (string memory) {
//        return "This is a vary vary long string that is longer than 32 bytes";
//    }

    //    function get_state() public view returns (string memory) {
    //        return state;
    //    }

    //    function set_state(string memory s) public {
    //        state = s;
    //    }
}
