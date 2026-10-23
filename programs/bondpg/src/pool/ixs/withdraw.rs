use crate::{
    constants::{NATIVE_MINT_STR, VAULT_SEED}, 
    error::PumpFunError, 
    MainState,
    PoolState
};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, CloseAccount, Mint, Token, TokenAccount, Transfer},
};
use std::str::FromStr;


pub fn withdraw(ctx: Context<AWithdrawState>) -> Result<()> {
    let admin = ctx.accounts.admin.to_account_info();
    
    let main_state = &ctx.accounts.main_state;

    require!(main_state.initialized.eq(&true), PumpFunError::Uninitialized);
    let pool_state = &ctx.accounts.pool_state;
    require!(pool_state.complete.eq(&true), PumpFunError::BondingCurveIncomplete);

    let admin_base_ata = ctx.accounts.admin_base_ata.to_account_info();
    let token_program = ctx.accounts.token_program.to_account_info();

    // send tokens in pool and virt
    let pool_base_transfer_cpi_account = Transfer{
        from: ctx.accounts.reserver_base_ata.to_account_info(),
        to: admin_base_ata.clone(),
        authority: pool_state.to_account_info()
    };
    token::transfer(CpiContext::new_with_signer(token_program.clone(), pool_base_transfer_cpi_account, &[&[
        PoolState::PREFIX_SEED,
        pool_state.base_mint.as_ref(),
        &[ctx.bumps["pool_state"]]
    ]]), pool_state.virt_base_reserves + pool_state.real_base_reserves - pool_state.real_utility_team_base_reserves)?;

    // send SOL in pool
    system_program::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(), 
            system_program::Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: admin,
            },
            &[&[
                VAULT_SEED,
                pool_state.key().as_ref(),
                &[ctx.bumps["vault"]]
            ]]
        ),
        pool_state.real_quote_reserves
    )?;
    
    Ok(())
}

#[derive(Accounts)]
pub struct AWithdrawState<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [MainState::PREFIX_SEED],
        bump,
    )]
    pub main_state: Box<Account<'info, MainState>>,

    #[account(
        mut,
        seeds = [VAULT_SEED, pool_state.key().as_ref()],
        bump
    )]
    /// CHECK:
    pub vault: AccountInfo<'info>,

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
    )]
    pub base_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = base_mint,
        associated_token::authority = pool_state,
    )]
    pub reserver_base_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = base_mint,
        associated_token::authority = admin,
    )]
    pub admin_base_ata: Box<Account<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
