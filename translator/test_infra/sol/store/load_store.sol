pragma solidity ^0.8.0;

contract load_store {
    struct TestStruct {
        uint a;
        uint b;
        bool c;
    }

    TestStruct struct1;
    bool flag;


    function set_all(uint a, uint b, bool c, bool f) public {
        struct1.a = a;
        struct1.b = b;
        struct1.c = c;
        flag = f;
    }

    function set_a(uint a) public {
        struct1.a = a;
    }

    function set_b(uint b) public {
        struct1.b = b;
    }

    function set_c(bool c) public {
        struct1.c = c;
    }

    function set_f(bool f) public {
        flag = f;
    }

    function get_all() public view returns (uint, uint, bool, bool) {
        return (struct1.a, struct1.b, struct1.c, flag);
    }

    function get_a() public view returns (uint) {
        return struct1.a;
    }

    function get_b() public view returns (uint) {
        return struct1.b;
    }

    function get_c() public view returns (bool) {
        return struct1.c;
    }

    function get_flag() public view returns (bool) {
        return flag;
    }
}
