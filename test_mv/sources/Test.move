module 0x1::Test {
    public fun test(): u64 {
        let x = 1000;
        let y = 1000;
        y = x * y;
        return y
    }
}
