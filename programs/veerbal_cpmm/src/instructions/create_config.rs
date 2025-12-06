use crate::{error::ErrorCode, AmmConfig};
use anchor_lang::prelude::*;

pub const CONFIG_SEED: &[u8] = b"AMM_CONFIG";

#[derive(Accounts)]
#[instruction(index: u8)]
pub struct CreateAmmConfig<'info> {
    #[account(mut, address = crate::ADMIN @ ErrorCode::InvalidOwner)]
    pub owner: Signer<'info>,

    #[account(init, seeds=[CONFIG_SEED, index.to_be_bytes().as_ref()], bump, payer = owner, space = AmmConfig::LEN)]
    pub amm_config: Account<'info, AmmConfig>,

    pub system_program: Program<'info, System>,
}

pub fn create_amm_config(
    ctx: Context<CreateAmmConfig>,
    index: u16,
    trade_fee_rate: u64,
    creator_fee_rate: u64,
    protocol_fee_rate: u64,
    fund_fee_rate: u64,
    create_pool_fee: u64,
    fund_owner: Pubkey,
) -> Result<()> {
    require!(
        trade_fee_rate + creator_fee_rate < 1_000_000,
        ErrorCode::FeeExceedHundredPercentage
    );

    require!(
        protocol_fee_rate + fund_fee_rate < 1_000_000,
        ErrorCode::FeeExceedHundredPercentage
    );

    let amm_config = &mut ctx.accounts.amm_config;
    amm_config.bump = ctx.bumps.amm_config;
    amm_config.index = index;

    amm_config.create_pool_fee = create_pool_fee;
    amm_config.creator_fee_rate = creator_fee_rate;
    amm_config.protocol_fee_rate = protocol_fee_rate;
    amm_config.fund_fee_rate = fund_fee_rate;
    amm_config.trade_fee_rate = trade_fee_rate;
    amm_config.protocol_owner = ctx.accounts.owner.key();
    amm_config.fund_owner = fund_owner;

    amm_config.disable_create_pool = false;

    Ok(())
}
