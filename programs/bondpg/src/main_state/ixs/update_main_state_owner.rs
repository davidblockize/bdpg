use crate::{error::PumpFunError, MainState};
use anchor_lang::prelude::*;

#[derive(AnchorDeserialize, AnchorSerialize, Debug, Clone, Copy)]
pub struct UpdateMainStateInput {
    trading_fee: u64,
}

pub fn update_main_state(
    ctx: Context<AUpdateMainState>,
    input: UpdateMainStateInput,
) -> Result<()> {
    let state = &mut ctx.accounts.main_state;
    require!(state.initialized.eq(&true), PumpFunError::Uninitialized);

    let owner = ctx.accounts.owner.to_account_info();
    require!(state.owner.eq(&owner.key()), PumpFunError::NoAdmin);

    state.owner = ctx.accounts.new_owner.key();
    state.fee_recipient = ctx.accounts.fee_recipient.key();
    state.trading_fee = input.trading_fee;
    
    Ok(())
}

#[derive(Accounts)]
pub struct AUpdateMainState<'info> {
    #[account(mut, address = main_state.owner @ PumpFunError::Unauthorised)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [MainState::PREFIX_SEED],
        bump,
        has_one = owner,
    )]
    pub main_state: Account<'info, MainState>,

    #[account(
        mut
    )]
    /// CHECK:
    pub fee_recipient: AccountInfo<'info>,

    #[account(
        mut
    )]
    /// CHECK:
    pub new_owner: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}
