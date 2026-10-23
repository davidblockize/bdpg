use std::str::FromStr;

use anchor_lang::solana_program::pubkey::Pubkey;

pub const NATIVE_MINT_STR: &'static str = "So11111111111111111111111111111111111111112";

pub const FEE_PER_DIV: u128 = 1000;

pub const VIRT_SOL_RESERVE: u64 = 28_000_000_000;
pub const REAL_SOL_THRESHOLD: u64 = 85_000_000_000;
pub const REAL_SOL_THRESHOLD_BUF: u64 = 1_000_000_000;

pub const USER_SEED: &[u8] = b"BONDPG_BONDING_USER_INFO_SEED";
pub const VAULT_SEED: &[u8] = b"BONDPG_BONDING_VAULT_SEED";
