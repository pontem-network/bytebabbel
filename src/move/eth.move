module 0x1::EthOpcodes {
	public fun addmod(x: u64, y: u64, m: u64): u64 {
		(x + y) % m
	}

	public fun mulmod(x: u64, y: u64, m: u64): u64 {
		(x * y) % m
	}
}
