use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{AmmConfig, constants::{AUTH_SEED, LP_MINT_SEED, POOL_SEED}, error::ErrorCode, instructions::CONFIG_SEED, states::PoolState};

// Account required
#[derive(Accounts)]
pub struct Deposit<'info> {
    // Person who is adding liquidity
    #[account(mut)]
    pub signer: Signer<'info>,

    // Above accounts dependent on pool state for updating supply, need pool state address for PDA derivation etc.
    #[account(mut, seeds=[POOL_SEED, amm_config.key().as_ref(), pool_state.token_0_mint.key().as_ref(), pool_state.token_1_mint.key().as_ref()], bump = pool_state.bump)]
    pub pool_state: Account<'info, PoolState>,

    // User will provide these accounts
    #[account(mut, token::mint = pool_state.token_0_mint, token::authority = signer)]
    pub signer_token_0: Account<'info, TokenAccount>,

    #[account(mut, token::mint = pool_state.token_1_mint, token::authority = signer)]
    pub signer_token_1: Account<'info, TokenAccount>,

    #[account(mut, constraint = token_0_vault.key() == pool_state.token_0_vault @ ErrorCode::InvalidVault, token::mint = pool_state.token_0_mint, token::authority = authority)]
    pub token_0_vault: Account<'info, TokenAccount>,

    #[account(mut, constraint = token_1_vault.key() == pool_state.token_1_vault @ ErrorCode::InvalidVault, token::mint = pool_state.token_1_mint, token::authority = authority)]
    pub token_1_vault: Account<'info, TokenAccount>,

    #[account(init_if_needed, associated_token::mint = lp_mint, associated_token::authority = signer, payer = signer)]
    pub signer_lp: Account<'info, TokenAccount>,

    #[account(mut, seeds=[LP_MINT_SEED, pool_state.key().as_ref()], bump = pool_state.mint_bump)]
    pub lp_mint: Account<'info, Mint>,

    // pool state dependent upon amm config
    #[account(seeds=[CONFIG_SEED, amm_config.index.to_be_bytes().as_ref()], bump = amm_config.bump)]
    pub amm_config: Account<'info, AmmConfig>,

    // Authority pda is needed to sign vault transactions
    #[account(seeds=[AUTH_SEED], bump = pool_state.auth_bump)]
    pub authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
