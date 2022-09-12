module 0x42::Users {
struct DU256 has copy, drop, store {
    v0: u64,
    v1: u64,
    v2: u64,
    v3: u64,
    v4: u64,
    v5: u64,
    v6: u64,
    v7: u64
}

struct Event has drop, store {
    data: vector<u8>,
    topics: vector<U256>
}
struct Memory has copy, drop, store {
    data: vector<u8>,
    effective_len: u64,
    limit: u64
}
struct Persist has store, key {
    tbl: Table<U256, U256>,
    events: EventHandle<Event>
}
struct U256 has copy, drop, store {
    v0: u64,
    v1: u64,
    v2: u64,
    v3: u64
}

    public fun get_id(_Arg0: &signer, _Arg1: vector<u8>): vector<u8> {
        return std::vector::empty<u8>()
    }

 public fun is_owner(_Arg0: &signer, _Arg1: vector<u8>): vector<u8> {
     return std::vector::empty<u8>()
 }
 public fun transfer(Arg0: &signer, Arg1: vector<u8>): vector<u8> {
     return std::vector::empty<u8>()
 }
 public fun create_user(Arg0: &signer, Arg1: vector<u8>): vector<u8> {
     return std::vector::empty<u8>()
 }
 public fun get_balance(Arg0: &signer, Arg1: vector<u8>): vector<u8> {
     return std::vector::empty<u8>()
 }
 public fun constructor(Arg0: &signer) {
     return std::vector::empty<u8>()
 }
}