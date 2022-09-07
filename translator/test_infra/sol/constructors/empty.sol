pragma solidity ^0.8.0;

contract empty {
    uint val;

    constructor() {
        if (true) {
            val = 1;
            return;
        } else {
            val = 2;
            return;
        }
    }

    function get_val() public view returns (uint256) {
        return val;
    }
}
