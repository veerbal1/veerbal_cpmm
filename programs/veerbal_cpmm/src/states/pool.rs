use anchor_lang::prelude::*;

pub enum PoolStatusBitIndex {
    Deposit,
    Withdraw,
    Swap,
}

#[account]
#[derive(InitSpace)]
pub struct PoolState {
    pub amm_config: Pubkey,

    pub pool_creator: Pubkey,

    pub token_0_mint: Pubkey,

    pub token_1_mint: Pubkey,

    pub token_0_vault: Pubkey,

    pub token_1_vault: Pubkey,

    pub token_0_program: Pubkey,

    pub token_1_program: Pubkey,

    pub lp_mint: Pubkey,

    pub lp_supply: u64,

    pub bump: u8,

    pub auth_bump: u8,

    pub token_0_bump: u8,

    pub token_1_bump: u8,

    pub mint_bump: u8,

    pub status: u8,

    pub open_time: u64,

    pub recent_epoch: u64,

    pub observation_key: Pubkey,

    pub protocol_token_0_fee: u64,
    pub protocol_token_1_fee: u64,

    pub fund_token_0_fee: u64,
    pub fund_token_1_fee: u64,

    pub creator_fee_on: u8,
    pub creator_fee_active: bool,
    pub creator_token_0_fee: u64,
    pub creator_token_1_fee: u64,

    pub mint_0_decimals: u8,
    pub mint_1_decimals: u8,
    pub lp_mint_decimals: u8,
}

impl PoolState {
    pub const LEN: usize = 8 + Self::INIT_SPACE;

    pub fn is_enabled(&self, action: PoolStatusBitIndex) -> bool {
        let mask = (1 as u8) << (action as u8);
        self.status & mask == 0
    }
}
