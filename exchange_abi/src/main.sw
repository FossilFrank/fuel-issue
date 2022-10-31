library exchange_abi;

use std::u128::U128;

pub struct RemoveLiquidityInfo {
    token_0_amount: u64,
    token_1_amount: u64,
}

pub struct PositionInfo {
    token_0_amount: u64,
    token_1_amount: u64,
}

pub struct PoolInfo {
    token_0_reserve: u64,
    token_1_reserve: u64,
    lp_token_supply: u64,
    twap_buffer_size: u64,
}

pub struct PreviewInfo {
    amount: u64,
    has_liquidity: bool,
}

pub struct Observation {
    timestamp: u64,
    price0CumulativeLast: U128,
    price1CumulativeLast: U128,
}

abi Exchange {
    ////////////////////
    // Read only
    ////////////////////
    /// Get information on the liquidity pool.
    #[storage(read)]
    fn get_pool_info() -> PoolInfo;
    /// Get information on the liquidity pool.
    #[storage(read)]
    fn get_add_liquidity_token_amount(token_0_amount: u64) -> u64;
    /// Get the minimum amount of coins that will be received for a swap_with_minimum.
    #[storage(read)]
    fn get_swap_with_minimum(amount: u64) -> PreviewInfo;
    /// Get required amount of coins for a swap_with_maximum.
    #[storage(read)]
    fn get_swap_with_maximum(amount: u64) -> PreviewInfo;
    /// Get the two tokens held in the pool
    #[storage(read)]
    fn get_tokens() -> (b256, b256);
    ////////////////////
    // Actions
    ////////////////////
    /// Deposit ETH and Tokens at current ratio to mint SWAYSWAP tokens.
    #[storage(read, write)]
    fn add_liquidity(recipient: Identity) -> u64;
    /// Burn SWAYSWAP tokens to withdraw ETH and Tokens at current ratio.
    #[storage(read, write)]
    fn remove_liquidity(recipient: Identity) -> RemoveLiquidityInfo;
    #[storage(read, write)]
    fn swap(amount_0_out: u64, amount_1_out: u64, recipient: Identity);
    /// Increase the size of the TWAP buffer to the given size
    #[storage(read, write)]
    fn expand_twap_buffer(new_slots: u64);
}
