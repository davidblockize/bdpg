use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct UserInfo {
    // Buy quote amount
    pub buy_quote_amount: u64,
    // Buy quote fee amount
    pub buy_quote_fee_amount: u64,
    // Buy token amount
    pub buy_token_amount: u64,
} 