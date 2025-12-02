use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct AmmConfig {
    pub bump: u8,

    pub index: u8,

    pub trade_fee_rate: u64,
    pub protocol_fee_rate: u64,
    pub fund_fee_rate: u64,
    pub creator_fee_rate: u64,
    pub create_pool_fee: u64,

    pub protocol_owner: Pubkey,
    pub fund_owner: Pubkey,
    pub disable_create_pool: bool,
}

impl AmmConfig {
    pub const LEN: usize = 8 + Self::INIT_SPACE;
}
