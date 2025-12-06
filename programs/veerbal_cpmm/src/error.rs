use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("InvalidOwner")]
    InvalidOwner,

    #[msg("FEE_EXCEED_HUNDRED_PERCENTAGE")]
    FeeExceedHundredPercentage,

    #[msg("Pool Creation disabled")]
    PoolCreationDisabled,

    #[msg("Insufficient Tokens to Mint")]
    InsufficientTokensToMint,
}
