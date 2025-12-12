use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::{
    AmmConfig, constants::{AUTH_SEED, POOL_SEED}, curve::swap_base_input_without_fees, error::ErrorCode, instructions::CONFIG_SEED, states::{PoolState, PoolStatusBitIndex}
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
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(seeds=[AUTH_SEED], bump = pool_state.auth_bump)]
    pub authority: UncheckedAccount<'info>,

    // Programs
    pub token_program: Program<'info, Token>,
}


pub fn swap(ctx: Context<Swap>, amount_in: u64, minimum_amount_out: u64) -> Result<()> {
    let pool_state= &mut ctx.accounts.pool_state;
    require!(pool_state.is_enabled(PoolStatusBitIndex::Swap), ErrorCode::SwapBlocked);
    require!(pool_state.open_time < Clock::get()?.unix_timestamp as u64, ErrorCode::SwapBlocked);

    require!(amount_in > 0, ErrorCode::InvalidTokenAmount);
    require!(minimum_amount_out > 0, ErrorCode::InvalidTokenAmount);

    let input_vault_balance = ctx.accounts.input_vault.amount as u128;
    let output_vault_balance = ctx.accounts.output_vault.amount as u128;

    let output_amount = swap_base_input_without_fees(amount_in as u128, input_vault_balance, output_vault_balance).ok_or(ErrorCode::MathOverflow)?;

    require!(output_amount > 0, ErrorCode::ZeroTradingTokens);
    let output_amount: u64 = output_amount.try_into().map_err(|_| ErrorCode::MathOverflow)?;
    require!(output_amount >= minimum_amount_out, ErrorCode::SlippageExceeded);


    // Transfer IN: user → input_vault
    let transfer_in_accounts = Transfer {
        from: ctx.accounts.input_token_account.to_account_info(),
        to: ctx.accounts.input_vault.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_in_accounts);
    token::transfer(cpi_ctx, amount_in)?;

    // Transfer OUT: output_vault → user (PDA signs!)
    let transfer_out_accounts = Transfer {
        from: ctx.accounts.output_vault.to_account_info(),
        to: ctx.accounts.output_token_account.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };

    let seeds = &[AUTH_SEED, &[pool_state.auth_bump]];
    let signer_seeds = &[&seeds[..]];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer_out_accounts,
        signer_seeds,
    );

    token::transfer(cpi_ctx, output_amount)?;

    Ok(())
}


