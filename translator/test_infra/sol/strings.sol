pragma solidity ^0.8.0;

contract Strings {
    string state = "Hello, World!";

    function const_str() public pure returns (string memory) {
        return "hello";
    }

    //    function get_state() public view returns (string memory) {
    //        return state;
    //    }

    //    function set_state(string memory s) public {
    //        state = s;
    //    }
}
