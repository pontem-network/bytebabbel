pragma solidity ^0.8.0;

contract with_data {
    uint256 val;

    constructor(uint init_val) public {
        val = init_val;
    }

    function get_val() public view returns (uint256) {
        return val;
    }
}
