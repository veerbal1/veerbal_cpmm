use crate::{
    constants::{AUTH_SEED, POOL_SEED, VAULT_SEED},
    error::ErrorCode,
    states::PoolState,
    AmmConfig,
};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
#[derive(Accounts)]
pub struct CollectCreatorFee<'info> {
    #[account(address = pool_state.token_0_mint)]
    pub token_0_mint: Account<'info, Mint>,

    #[account(address = pool_state.token_1_mint)]
    pub token_1_mint: Account<'info, Mint>,

    #[account(mut, seeds=[POOL_SEED, pool_state.amm_config.key().as_ref(), token_0_mint.key().as_ref(), token_1_mint.key().as_ref()], bump = pool_state.bump)]
    pub pool_state: Box<Account<'info, PoolState>>,

    #[account(mut, seeds=[VAULT_SEED, pool_state.key().as_ref(), token_0_mint.key().as_ref()], bump = pool_state.token_0_bump)]
    pub token_0_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut, seeds=[VAULT_SEED, pool_state.key().as_ref(), token_1_mint.key().as_ref()], bump = pool_state.token_1_bump)]
    pub token_1_vault: Box<Account<'info, TokenAccount>>,

    /// CHECKED - No deserialization
    #[account(seeds=[AUTH_SEED], bump = pool_state.auth_bump)]
    pub authority: UncheckedAccount<'info>,

    #[account(mut, associated_token::mint = token_0_mint, associated_token::authority = owner)]
    pub receiver_token_0_account: Box<Account<'info, TokenAccount>>,

    #[account(mut, associated_token::mint = token_1_mint, associated_token::authority = owner)]
    pub receiver_token_1_account: Box<Account<'info, TokenAccount>>,

    #[account(mut, address = pool_state.pool_creator @ ErrorCode::InvalidOwner)]
    pub owner: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>
}

pub fn collect_creator_fee(ctx: Context<CollectCreatorFee>) -> Result<()> {
    // TODO: Logic here
    Ok(())
}
