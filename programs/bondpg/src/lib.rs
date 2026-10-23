#![allow(unused)]

use anchor_lang::prelude::*;

pub mod main_state;
pub mod pool;

pub mod constants;
pub mod error;
pub mod utils;

use main_state::*;
use pool::*;

declare_id!("8j8xK5B6fQZkP4esSe7KZG1LnTfGxuBuVKUXiedgEpWz");

#[program]
pub mod bondpg {
    use super::*;

    pub fn init_main_state(ctx: Context<AInitMainState>) -> Result<()> {
        main_state::init_main_state(ctx)
    }

    pub fn update_main_state(ctx: Context<AUpdateMainState>, input: UpdateMainStateInput) -> Result<()> {
        main_state::update_main_state(ctx, input)
    }

    pub fn create_pool(ctx: Context<ACreatePool>, input: CreatePoolInput) -> Result<()> {
        pool::create_pool(ctx, input)
    }

    pub fn buy(ctx: Context<ABuy>, input: BuyInput) -> Result<()> {
        pool::buy(ctx, input)
    }

    pub fn claim(ctx: Context<ClaimState>) -> Result<()> {
        pool::claim(ctx)
    }

    pub fn withdraw(ctx: Context<AWithdrawState>) -> Result<()> {
        pool::withdraw(ctx)
    }
}
