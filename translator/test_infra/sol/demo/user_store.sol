pragma solidity ^0.8.0;

contract UserStore {
    struct User {
        uint256 id;
        string name;
        bool is_admin;
    }

    uint256 seq;
    mapping(address => User) user_store;

    constructor(address admin_address, string memory name) {
        user_store[admin_address] = User(seq, name, true);
    }

    function create_user(string memory name) public returns (uint256) {
        user_store[msg.sender] = User(seq, name, false);
        seq = seq + 1;
        return seq;
    }

    function get_name() public view returns (string memory) {
        return user_store[msg.sender].name;
    }

    function get_id() public view returns (uint256) {
        return user_store[msg.sender].id;
    }

    function is_admin() public view returns (bool) {
        return user_store[msg.sender].is_admin;
    }
}
