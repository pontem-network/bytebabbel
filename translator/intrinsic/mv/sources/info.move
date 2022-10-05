module self::info {
    use aptos_framework::block;
    use self::u256::{U256, from_u64};

    public fun get_current_block_height(): U256 {
        from_u64(block::get_current_block_height())
    }

    public fun get_epoch_interval_secs(): U256 {
        from_u64(block::get_epoch_interval_secs())
    }
}