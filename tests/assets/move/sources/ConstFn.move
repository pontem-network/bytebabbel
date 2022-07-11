module 0x1::ConstFn {
    public fun const_fun(a: u128, b: u128): u128 {
        let g = 10;
        g = g + a;
        return a + g + b
    }
}
