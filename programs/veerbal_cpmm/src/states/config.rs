use anchor_lang::prelude::*;

pub const AMM_CONFIG_SEED: &str = "amm_config";

#[account]
pub struct AmmConfig {
    pub bump: u8,

    pub disable_create_pool: bool,

    pub index: u16,

    pub trade_fee_rate: u64,

    pub protocol_fee_rate: u64,

    pub fund_fee_rate: u64,

    pub create_pool_fee: u64,

    pub protocol_owner: Pubkey, 

    pub fund_owner: Pubkey,

    pub creator_fee_rate: u64,

    pub padding: [u64; 15]
}

impl AmmConfig {
    pub const LEN: usize = 8 // Discriminator
    + 1 // bump
    + 1 // bool
    + 2 // u16
    + 8 + 8 + 8 + 8 // u64
    + 32 + 32 // pubkey
    + 8 //u64
    + (8*15);
}