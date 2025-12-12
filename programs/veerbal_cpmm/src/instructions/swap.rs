use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::{
    constants::{AUTH_SEED, POOL_SEED},
    curve::{creator_fee, fund_fee, protocol_fee, swap_base_input_without_fees, trade_fee},
    error::ErrorCode,
    states::{PoolState, PoolStatusBitIndex},
    AmmConfig,
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
    let pool_state = &mut ctx.accounts.pool_state;
    require!(
        pool_state.is_enabled(PoolStatusBitIndex::Swap),
        ErrorCode::SwapBlocked
    );
    require!(
        pool_state.open_time < Clock::get()?.unix_timestamp as u64,
        ErrorCode::SwapBlocked
    );
    require!(amount_in > 0, ErrorCode::InvalidTokenAmount);
    require!(minimum_amount_out > 0, ErrorCode::InvalidTokenAmount);
    // Step 1: Determine swap direction
    let is_token_0_input = ctx.accounts.input_vault.key() == pool_state.token_0_vault;
    // Step 2: Get vault amounts in correct order (vault_0, vault_1)
    let (vault_0_amount, vault_1_amount) = if is_token_0_input {
        (
            ctx.accounts.input_vault.amount,
            ctx.accounts.output_vault.amount,
        )
    } else {
        (
            ctx.accounts.output_vault.amount,
            ctx.accounts.input_vault.amount,
        )
    };
    // Step 3: Subtract accumulated fees
    let (clean_vault_0, clean_vault_1) =
        pool_state.vault_amount_without_fee(vault_0_amount, vault_1_amount)?;
    // Step 4: Map back to input/output for swap formula
    let (input_vault_balance, output_vault_balance) = if is_token_0_input {
        (clean_vault_0 as u128, clean_vault_1 as u128)
    } else {
        (clean_vault_1 as u128, clean_vault_0 as u128)
    };

    // k verification: constant BEFORE swap
    let constant_before = input_vault_balance
        .checked_mul(output_vault_balance)
        .ok_or(ErrorCode::MathOverflow)?;

    // Calculate trade fee
    let fee = trade_fee(amount_in as u128, ctx.accounts.amm_config.trade_fee_rate)
        .ok_or(ErrorCode::MathOverflow)?;
    let actual_input: u128 = (amount_in as u128)
        .checked_sub(fee)
        .ok_or(ErrorCode::MathOverflow)?;
    // Split fee into protocol/fund/creator
    let protocol_fee_amount = protocol_fee(fee, ctx.accounts.amm_config.protocol_fee_rate)
        .ok_or(ErrorCode::MathOverflow)?;
    let fund_fee_amount =
        fund_fee(fee, ctx.accounts.amm_config.fund_fee_rate).ok_or(ErrorCode::MathOverflow)?;
    let creator_fee_amount = creator_fee(fee, ctx.accounts.amm_config.creator_fee_rate)
        .ok_or(ErrorCode::MathOverflow)?;
    // Track fees in pool_state
    if is_token_0_input {
        pool_state.protocol_token_0_fee = pool_state
            .protocol_token_0_fee
            .checked_add(protocol_fee_amount as u64)
            .ok_or(ErrorCode::MathOverflow)?;
        pool_state.fund_token_0_fee = pool_state
            .fund_token_0_fee
            .checked_add(fund_fee_amount as u64)
            .ok_or(ErrorCode::MathOverflow)?;
        pool_state.creator_token_0_fee = pool_state
            .creator_token_0_fee
            .checked_add(creator_fee_amount as u64)
            .ok_or(ErrorCode::MathOverflow)?;
    } else {
        pool_state.protocol_token_1_fee = pool_state
            .protocol_token_1_fee
            .checked_add(protocol_fee_amount as u64)
            .ok_or(ErrorCode::MathOverflow)?;
        pool_state.fund_token_1_fee = pool_state
            .fund_token_1_fee
            .checked_add(fund_fee_amount as u64)
            .ok_or(ErrorCode::MathOverflow)?;
        pool_state.creator_token_1_fee = pool_state
            .creator_token_1_fee
            .checked_add(creator_fee_amount as u64)
            .ok_or(ErrorCode::MathOverflow)?;
    }
    // Calculate output using x*y=k formula
    let output_amount =
        swap_base_input_without_fees(actual_input, input_vault_balance, output_vault_balance)
            .ok_or(ErrorCode::MathOverflow)?;
    require!(output_amount > 0, ErrorCode::ZeroTradingTokens);
    let output_amount: u64 = output_amount
        .try_into()
        .map_err(|_| ErrorCode::MathOverflow)?;
    require!(
        output_amount >= minimum_amount_out,
        ErrorCode::SlippageExceeded
    );

    let new_input_balance = input_vault_balance
        .checked_add(actual_input)
        .ok_or(ErrorCode::MathOverflow)?;
    let new_output_balance = output_vault_balance
        .checked_sub(output_amount as u128)
        .ok_or(ErrorCode::MathOverflow)?;
    let constant_after = new_input_balance
        .checked_mul(new_output_balance)
        .ok_or(ErrorCode::MathOverflow)?;

    require!(
        constant_after >= constant_before,
        ErrorCode::ConstantProductInvariant
    );

    // Transfer IN: user → input_vault
    let transfer_in_accounts = Transfer {
        from: ctx.accounts.input_token_account.to_account_info(),
        to: ctx.accounts.input_vault.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_in_accounts,
    );
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
