use anchor_lang::{prelude::*, system_program};
use anchor_spl::{associated_token::AssociatedToken, token::{self,Mint, Token, TokenAccount, Transfer}};
use crate::{
    constants::{REAL_SOL_THRESHOLD, REAL_SOL_THRESHOLD_BUF, USER_SEED, VAULT_SEED}, error::PumpFunError, main_state, pool, utils::calculate_trading_fee, CompleteEvent, MainState, PoolState, TradeEvent, UserInfo
};

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, Debug)]
pub struct BuyInput {
    pub amount: u64,
    pub cpn: bool,
    pub infbene: u64,
    pub userbene: u64,
}

pub fn buy(ctx:Context<ABuy>, input: BuyInput)->Result<()>{
    let main_state = &mut ctx.accounts.main_state;
    require!(main_state.initialized.eq(&true), PumpFunError::Uninitialized);
    
    let pool_state = &mut ctx.accounts.pool_state;
    require!(pool_state.complete.eq(&false), PumpFunError::BondingCurveComplete);

    let current_time = Clock::get()?.unix_timestamp;
    require!(pool_state.end_time > current_time, PumpFunError::BondingCurveExpired);

    let mut calculated_input_amount = input.amount;
    let mut remain_real_quote_reserves_amount = pool_state.real_sol_threshold - pool_state.real_quote_reserves + calculate_trading_fee(main_state.trading_fee, pool_state.real_sol_threshold - pool_state.real_quote_reserves) + REAL_SOL_THRESHOLD_BUF;
    if (input.amount > remain_real_quote_reserves_amount) {
        calculated_input_amount = remain_real_quote_reserves_amount;
    }

    let buyer = ctx.accounts.buyer.to_account_info();
    let buyer_base_ata = &ctx.accounts.buyer_base_ata;
    let token_program = ctx.accounts.token_program.to_account_info();
    let system_program = ctx.accounts.system_program.to_account_info();

    let user_info = &mut ctx.accounts.user_info;

    let mut trading_fee = main_state.trading_fee;
    let total_bene = input.infbene + input.userbene;
    let mut inf_fee = 0;
    if (input.cpn && total_bene <= 50) {
        let user_trading_fee = main_state.trading_fee * input.userbene / 100;
        let inf_trading_fee = main_state.trading_fee * input.infbene / 100;
        trading_fee = main_state.trading_fee - user_trading_fee;

        inf_fee = calculate_trading_fee(inf_trading_fee, calculated_input_amount);
    }
    
    let fee = calculate_trading_fee(trading_fee, calculated_input_amount);
    let input_amount = calculated_input_amount - fee;
    let output_amount = pool_state.compute_receivable_amount_on_buy(input_amount);

    user_info.buy_quote_amount = user_info.buy_quote_amount + input_amount;
    user_info.buy_quote_fee_amount = user_info.buy_quote_fee_amount + fee;
    user_info.buy_token_amount = user_info.buy_token_amount + output_amount;
    require!(user_info.buy_token_amount <= (pool_state.total_token_supply / 10), PumpFunError::ExceedMaxBuyAmount);

    let mut fee_amount = fee;
    if inf_fee > 0 {
        fee_amount = fee - inf_fee;
    }

    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(), 
            system_program::Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.fee_recipient.to_account_info(),
            })
        , fee_amount
    )?;

    if (inf_fee > 0) {
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(), 
                system_program::Transfer {
                    from: ctx.accounts.buyer.to_account_info(),
                    to: ctx.accounts.inf.to_account_info(),
                })
            , inf_fee
        )?; 
    }

    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(), 
            system_program::Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            })
        , input_amount
    )?;

    // sending tokens from reserve ata (meme)
    let output_amount_transfer_cpi_account = Transfer{
        from: ctx.accounts.reserver_base_ata.to_account_info(),
        to: buyer_base_ata.to_account_info(),
        authority: pool_state.to_account_info()
    };
    token::transfer(CpiContext::new_with_signer(token_program.clone(), output_amount_transfer_cpi_account, &[&[
        PoolState::PREFIX_SEED,
        pool_state.base_mint.as_ref(),
        &[ctx.bumps["pool_state"]]
    ]]), output_amount)?;

    emit!(TradeEvent {
        user: buyer.key(), 
        base_mint: pool_state.base_mint, 
        token_amount: output_amount, 
        sol_amount: calculated_input_amount, 
        base_reserves: pool_state.real_base_reserves + pool_state.virt_base_reserves, 
        quote_reserves: pool_state.virt_quote_reserves + pool_state.real_quote_reserves, 
        is_buy: true, 
        timestamp: Clock::get()?.unix_timestamp,
        infbene: inf_fee,
        pfee: fee_amount,
        inf: ctx.accounts.inf.to_account_info().key(),
    });

    if (pool_state.real_quote_reserves >= pool_state.real_sol_threshold) {
        pool_state.complete = true;
        
        emit!(CompleteEvent {
            user: buyer.key(), 
            base_mint: pool_state.base_mint, 
            timestamp: Clock::get()?.unix_timestamp,
        });
    }

    Ok(())
}

#[derive(Accounts)]
#[instruction(input: BuyInput)]
pub struct ABuy<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    /// CHECK: this should be set by admin
    pub inf: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [MainState::PREFIX_SEED],
        bump,
    )]
    pub main_state: Box<Account<'info, MainState>>,
    
    #[account(mut, address = main_state.fee_recipient,)]
    /// CHECK: this should be set by admin
    pub fee_recipient: AccountInfo<'info>,

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
        associated_token::mint = base_mint,
        associated_token::authority = buyer,
    )]
    pub buyer_base_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = base_mint,
        associated_token::authority = pool_state,
    )]
    pub reserver_base_ata: Box<Account<'info, TokenAccount>>,

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
