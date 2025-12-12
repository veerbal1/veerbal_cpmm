use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    AmmConfig, constants::{AUTH_SEED, POOL_SEED}, error::ErrorCode, instructions::CONFIG_SEED, states::{PoolState, PoolStatusBitIndex}
};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut, seeds=[POOL_SEED, pool_state.amm_config.key().as_ref(), pool_state.token_0_mint.key().as_ref(), pool_state.token_1_mint.key().as_ref()], bump = pool_state.bump)]
    pub pool_state: Account<'info, PoolState>,

    #[account(address = pool_state.amm_config)]
    pub amm_config: Account<'info, AmmConfig>,

    #[account(address =  input_vault.mint @ ErrorCode::MintMismatch)]
    pub input_token_mint: Account<'info, Mint>,

    #[account(address = output_vault.mint @ ErrorCode::MintMismatch)]
    pub output_token_mint: Account<'info, Mint>,

    #[account(mut, constraint = input_vault.key() == pool_state.token_0_vault || input_vault.key() == pool_state.token_1_vault @ ErrorCode::InvalidVault,)]
    pub input_vault: Account<'info, TokenAccount>,

    #[account(mut,  constraint = output_vault.key() == pool_state.token_0_vault 
    || output_vault.key() == pool_state.token_1_vault @ ErrorCode::InvalidVault, constraint = output_vault.key() != input_vault.key() @ ErrorCode::SameVault)]
    pub output_vault: Account<'info, TokenAccount>,

    // User accounts
    #[account(mut, token::mint = input_token_mint, token::authority = signer)]
    pub input_token_account: Account<'info, TokenAccount>,

    #[account(mut, token::mint = output_token_mint, token::authority = signer)]
    pub output_token_account: Account<'info, TokenAccount>,

    // Authority
    #[account(seeds=[AUTH_SEED], bump = pool_state.auth_bump)]
    pub authority: UncheckedAccount<'info>,

    // Programs
    pub token_program: Program<'info, Token>,
}


pub fn swap(ctx: Context<Swap>) -> Result<()> {
    let pool_state= &mut ctx.accounts.pool_state;
    require!(pool_state.is_enabled(PoolStatusBitIndex::Swap), ErrorCode::SwapBlocked);
    require!(pool_state.open_time < Clock::get()?.unix_timestamp as u64, ErrorCode::SwapBlocked);


    Ok(())
}


