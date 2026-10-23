use crate::{
    constants::{VIRT_SOL_RESERVE},
    MainState,
    error::PumpFunError
};
use anchor_lang::prelude::*;
use std::str::FromStr;

pub fn init_main_state(ctx: Context<AInitMainState>) -> Result<()> {
    let state = &mut ctx.accounts.main_state;
    require!(state.initialized.eq(&false), PumpFunError::AlreadyInitialized);

    state.initialized = true;
    state.owner = ctx.accounts.owner.key();
    state.fee_recipient = ctx.accounts.owner.key();
    state.init_virt_quote_reserves = VIRT_SOL_RESERVE;    // default: 28 SOL
    state.trading_fee = 2_000;   // default: 2%
    Ok(())
}

#[derive(Accounts)]
pub struct AInitMainState<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init,
        payer = owner,
        seeds = [MainState::PREFIX_SEED],
        bump,
        space = 8 + MainState::MAX_SIZE
    )]
    pub main_state: Account<'info, MainState>,

    pub system_program: Program<'info, System>,
}
