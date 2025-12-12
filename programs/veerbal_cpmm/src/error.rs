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

    #[msg("Invalid Vault")]
    InvalidVault,

    #[msg("SameVault")]
    SameVault,

    #[msg("Deposits are currently disabled for this pool")]
    DepositDisabled,

    #[msg("LP amount should be greater than 0")]
    InvalidLPAmount,

    #[msg("Token Amount should be greater than 0")]
    InvalidTokenAmount,

    #[msg("Pool not initialized yet")]
    PoolNotInitialized,

    #[msg("Maximum amount exceed")]
    MaximumAmountExceed,

    #[msg("Math overflow")]
    MathOverflow,

    #[msg("Withdraws are current disabled for this pool")]
    WithdrawDisabled,

    #[msg("SlippageExceeded")]
    SlippageExceeded,

    #[msg("ZeroTradingTokens")]
    ZeroTradingTokens,

    #[msg("Mint Address Mismatch")]
    MintMismatch,

    #[msg("SwapBlocked")]
    SwapBlocked,

    #[msg("Constant product invariant violated")]
    ConstantProductInvariant,

    #[msg("Invalid Fee Receiver")]
    InvalidFeeReceiver,
}
