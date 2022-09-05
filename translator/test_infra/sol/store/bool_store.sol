pragma solidity ^0.8.0;

contract bool_store {
    bool flag;

    function store(bool f) public {
        flag = f;
    }

    function load() public view returns (bool) {
        return flag;
    }
}
