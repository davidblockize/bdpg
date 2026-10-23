use crate::{
    constants::NATIVE_MINT_STR,
    MainState, PoolState,
    CreateEvent,
    error::PumpFunError,
    utils::{check_balance_on_pool_creator, sync_native_amount},
    constants::VAULT_SEED,
    constants::REAL_SOL_THRESHOLD,
};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, SyncNative, Token, TokenAccount, Transfer},
};

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, Debug)]
pub struct CreatePoolInput {
    pub base_amount: u64,
    pub quote_amount: u64,
    pub end_time: i64,
    pub utility: bool,
    pub virt_sol_init_reserve: u64,//28sol * 10**9
    pub real_sol_threshold: u64,
    pub token_percent_sell_thresold: u64,
    pub token_percent_liq_thresold: u64,
}

pub fn create_pool(ctx: Context<ACreatePool>, input: CreatePoolInput) -> Result<()> {
    let main_state = &mut ctx.accounts.main_state;
    require!(main_state.initialized.eq(&true), PumpFunError::Uninitialized);

    let creator = ctx.accounts.creator.to_account_info();

    let pool_state = &mut ctx.accounts.pool_state;
    let system_program = ctx.accounts.system_program.to_account_info();
    let token_program = ctx.accounts.token_program.to_account_info();
    let creator_base_ata = &ctx.accounts.creator_base_ata;

    pool_state.owner = creator.key();
    pool_state.base_mint = creator_base_ata.mint;
    pool_state.total_token_supply = input.base_amount;
    pool_state.end_time = input.end_time;
    pool_state.real_quote_reserves = input.quote_amount;

    if (input.utility) {
        pool_state.real_base_reserves = input.token_percent_sell_thresold;
        pool_state.virt_base_reserves = input.base_amount - pool_state.real_base_reserves;
        pool_state.real_utility_team_base_reserves = input.base_amount - input.token_percent_sell_thresold - input.token_percent_liq_thresold;
        pool_state.virt_quote_reserves = input.virt_sol_init_reserve;
        pool_state.real_sol_threshold = input.real_sol_threshold;
    } else {
        pool_state.real_base_reserves = input.base_amount * 8 / 10;
        pool_state.virt_base_reserves = input.base_amount - pool_state.real_base_reserves;
        pool_state.real_utility_team_base_reserves = 0;
        pool_state.virt_quote_reserves = main_state.init_virt_quote_reserves;
        pool_state.real_sol_threshold = REAL_SOL_THRESHOLD;
    }

    pool_state.konst = (pool_state.real_base_reserves as u128)
        .checked_mul((pool_state.virt_quote_reserves + pool_state.real_quote_reserves) as u128)
        .unwrap();

    //transfer
    let base_transfer_cpi_accounts = Transfer {
        from: ctx.accounts.creator_base_ata.to_account_info(),
        to: ctx.accounts.reserver_base_ata.to_account_info(),
        authority: creator.clone(),
    };

    if (input.utility) {
        token::transfer(
            CpiContext::new(token_program.to_account_info(), base_transfer_cpi_accounts),
            input.token_percent_sell_thresold + input.token_percent_liq_thresold,
        )?;    
    } else {
        token::transfer(
            CpiContext::new(token_program.to_account_info(), base_transfer_cpi_accounts),
            input.base_amount,
        )?;
    }

    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(), 
            system_program::Transfer {
                from: ctx.accounts.creator.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            })
        , input.quote_amount
    )?;

    emit!(CreateEvent {
        creator: pool_state.owner, 
        base_mint: pool_state.base_mint, 
        base_reserves: pool_state.real_base_reserves + pool_state.virt_base_reserves, 
        quote_reserves: pool_state.virt_quote_reserves + pool_state.real_quote_reserves, 
        timestamp: Clock::get()?.unix_timestamp,
        total_token_supply: pool_state.total_token_supply
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(input: CreatePoolInput)]
pub struct ACreatePool<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [MainState::PREFIX_SEED],
        bump,
    )]
    pub main_state: Box<Account<'info, MainState>>,

    #[account(
        init,
        payer = creator,
        seeds =[
            PoolState::PREFIX_SEED,
            base_mint.key().as_ref(),
        ],
        bump,
        space = 8 + PoolState::MAX_SIZE
    )]
    pub pool_state: Box<Account<'info, PoolState>>,

    #[account(
        mut,
        seeds = [VAULT_SEED, pool_state.key().as_ref()],
        bump
    )]
    /// CHECK:
    pub vault: AccountInfo<'info>,

    pub base_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer=creator,
        associated_token::mint =base_mint,
        associated_token::authority = creator,
        constraint = check_balance_on_pool_creator(creator_base_ata.as_ref(), input.base_amount) @ PumpFunError::InsufficientFund
    )]
    pub creator_base_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer=creator,
        associated_token::mint = base_mint,
        associated_token::authority = pool_state,
    )]
    pub reserver_base_ata: Box<Account<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
