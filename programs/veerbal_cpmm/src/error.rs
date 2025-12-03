use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("InvalidOwner")]
    InvalidOwner,

    #[msg("FEE_EXCEED_HUNDRED_PERCENTAGE")]
    FeeExceedHundredPercentage,
}
