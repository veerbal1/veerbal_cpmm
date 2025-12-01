use anchor_lang::prelude::*;


pub const POOL_SEED: &str = "pool";
pub const POOL_LP_MINT_SEED: &str = "pool_lp_mint";
pub const POOL_VAULT_SEED: &str = "pool_vault";

pub enum PoolStatusBitIndex {
    Deposit,
    Withdraw,
    Swap,
}

pub enum PoolStatusBitFlag {
    Enable,
    Disable,
}

// pub struct SwapParams {
//     pub trade_direction: TradeDirection,
//     pub total_input_token_amount: u64,
//     pub total_output_token_amount: u64,
//     pub token_0_price_x64: u128,
//     pub token_1_price_x64: u128,
//     pub is_creator_fee_on_input: bool,
// }

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum CreatorFeeOn {
    BothToken,

    OnlyToken0,

    OnlyToken1
}

impl CreatorFeeOn {
    fn from_u8(value: u8) -> Result<Self> {
        match value {
            0 => Ok(CreatorFeeOn::BothToken),
            1 => Ok(CreatorFeeOn::OnlyToken0),
            2 => Ok(CreatorFeeOn::OnlyToken1),
            _ => panic!("Invalid Fee"),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            CreatorFeeOn::BothToken => 0u8,
            CreatorFeeOn::OnlyToken0 => 1u8,
            CreatorFeeOn::OnlyToken1 => 2u8,
        }
    }
}


#[account(zero_copy(unsafe))]
#[repr(C, packed)]
#[derive(Default, Debug)]
pub struct PoolState {
    pub amm_config: Pubkey,

    pub pool_creator: Pubkey,

    pub token_0_vault: Pubkey,

    pub token_1_vault: Pubkey,

    pub lp_mint: Pubkey,

    pub token_0_mint: Pubkey,

    pub token_1_mint: Pubkey,

    pub token_0_program: Pubkey,

    pub token_1_program: Pubkey,

    pub observation_key: Pubkey,

    pub auth_bump: u8,

    pub status: u8,

    pub lp_mint_decimals: u8,

    pub mint_0_decimals: u8,
    pub mint_1_decimals: u8,

    pub lp_supply: u64,

    pub protocol_fees_token_0: u64,
    pub protocol_fees_token_1: u64,

    pub fund_fees_token_0: u64,
    pub fund_fees_token_1: u64,

    pub open_time: u64,
    pub recent_epoch: u64,

    pub creator_fee_on: u8,
    pub enable_creator_fee: bool,
    pub padding1: [u8; 6],
    pub creator_fees_token_0: u64,
    pub creator_fees_token_1: u64,

    pub padding: [u64; 28]
}

impl PoolState {
    pub const LEN: usize = 8 + (10 * 32) + (5 * 1) + (7 * 8) + 1 + 1 + 6 + 8 + 8 + (8 * 28);
}