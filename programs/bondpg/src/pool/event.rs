use anchor_lang::prelude::*;

#[event]
pub struct CreateEvent {
    pub creator: Pubkey,
    pub base_mint: Pubkey,
    pub base_reserves: u64,
    pub quote_reserves: u64,
    pub timestamp: i64,
    pub total_token_supply: u64,
}

#[event]
pub struct TradeEvent {
    pub user: Pubkey,
    pub base_mint: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub base_reserves: u64,
    pub quote_reserves: u64,
    pub is_buy: bool,
    pub timestamp: i64,
    pub infbene: u64,
    pub pfee: u64,
    pub inf: Pubkey,
}

#[event]
pub struct CompleteEvent {
    pub user: Pubkey,
    pub base_mint: Pubkey,
    pub timestamp: i64,
}
