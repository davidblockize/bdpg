use anchor_lang::{prelude::*, system_program};
use anchor_spl::{associated_token::AssociatedToken, token::{self,Mint, Token}};
use crate::{
    constants::{USER_SEED, VAULT_SEED}, error::PumpFunError, main_state, MainState, PoolState, UserInfo
};

pub fn claim(ctx:Context<ClaimState>)->Result<()>{
    let main_state = &mut ctx.accounts.main_state;
    require!(main_state.initialized.eq(&true), PumpFunError::Uninitialized);
    
    let pool_state = &mut ctx.accounts.pool_state;
    require!(pool_state.complete.eq(&false), PumpFunError::BondingCurveComplete);

    let current_time = Clock::get()?.unix_timestamp;
    require!(pool_state.end_time < current_time, PumpFunError::BondingCurveIncomplete);

    let user_info = &mut ctx.accounts.user_info;
    let quote_amount = user_info.buy_quote_amount - user_info.buy_quote_fee_amount;

    require!(quote_amount > 0, PumpFunError::InsufficientFund);

    system_program::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(), 
            system_program::Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.buyer.to_account_info(),
            },
            &[&[
                VAULT_SEED,
                pool_state.key().as_ref(),
                &[ctx.bumps["vault"]]
            ]]
        ),
        quote_amount
    )?;

    user_info.buy_quote_amount = 0;
    user_info.buy_quote_fee_amount = 0;

    Ok(())
}

#[derive(Accounts)]
pub struct ClaimState<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(
        mut,
        seeds = [MainState::PREFIX_SEED],
        bump,
    )]
    pub main_state: Box<Account<'info, MainState>>,
    
    #[account(
        mut,
        seeds = [
            PoolState::PREFIX_SEED,
            base_mint.key().as_ref(), 
        ],
        bump,
    )]
    pub pool_state: Box<Account<'info, PoolState>>,

    #[account(
        mut,
        seeds = [VAULT_SEED, pool_state.key().as_ref()],
        bump
    )]
    /// CHECK:
    pub vault: AccountInfo<'info>,

    #[account(address = pool_state.base_mint)]
    pub base_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = buyer,
        space = 8 + std::mem::size_of::<UserInfo>(),
        seeds = [USER_SEED, pool_state.key().as_ref(), buyer.key().as_ref()],
        bump
    )]
    pub user_info: Box<Account<'info, UserInfo>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
