use anchor_lang::prelude::error_code;

#[error_code]
pub enum PumpFunError {
    #[msg("Uninitialized")]
    Uninitialized,

    #[msg("AlreadyInitialized")]
    AlreadyInitialized,

    #[msg("Unauthorised")]
    Unauthorised,

    #[msg("Insufficient fund")]
    InsufficientFund,

    #[msg("One token should be Sol")]
    UnknownToken,

    #[msg("BondingCurve incomplete")]
    BondingCurveIncomplete,

    #[msg("BondingCurve complete")]
    BondingCurveComplete,

    #[msg("BondingCurve expired")]
    BondingCurveExpired,

    #[msg("Exceed max buy amount")]
    ExceedMaxBuyAmount,

    #[msg("No admin")]
    NoAdmin,

    #[msg("No meme coin")]
    NoMemeCoin,
}
