module self::du256 {
    // DU256.
    //=================================================================================================================

    // Errors.
    /// When trying to get or put word into DU256 but it's out of index.
    const EWORDS_OVERFLOW: u64 = 1;

    // use self::u256::{U256, new_u256};

    /// Double `U256` used for multiple (to store overflow).
    struct DU256 has copy, drop, store {
        v0: u64,
        v1: u64,
        v2: u64,
        v3: u64,
        v4: u64,
        v5: u64,
        v6: u64,
        v7: u64,
    }

    public fun new_du256(v0: u64, v1: u64, v2: u64, v3: u64, v4: u64, v5: u64, v6: u64, v7: u64): DU256 {
        DU256 {
            v0,
            v1,
            v2,
            v3,
            v4,
            v5,
            v6,
            v7,
        }
    }

    /// Get word from `DU256` by index.
    public fun get_d(a: & DU256, i: u64): u64 {
        if (i == 0) {
            a.v0
        } else if (i == 1) {
            a.v1
        } else if (i == 2) {
            a.v2
        } else if (i == 3) {
            a.v3
        } else if (i == 4) {
            a.v4
        } else if (i == 5) {
            a.v5
        } else if (i == 6) {
            a.v6
        } else if (i == 7) {
            a.v7
        } else {
            abort EWORDS_OVERFLOW
        }
    }

    /// Put new word into `DU256` by index `i`.
    public fun put_d(a: &mut DU256, i: u64, val: u64) {
        if (i == 0) {
            a.v0 = val;
        } else if (i == 1) {
            a.v1 = val;
        } else if (i == 2) {
            a.v2 = val;
        } else if (i == 3) {
            a.v3 = val;
        } else if (i == 4) {
            a.v4 = val;
        } else if (i == 5) {
            a.v5 = val;
        } else if (i == 6) {
            a.v6 = val;
        } else if (i == 7) {
            a.v7 = val;
        } else {
            abort EWORDS_OVERFLOW
        }
    }

    public fun zero_d(): DU256 {
        DU256 {
            v0:0,
            v1:0,
            v2:0,
            v3:0,
            v4:0,
            v5:0,
            v6:0,
            v7:0,
        }
    }
}