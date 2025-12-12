use crate::{
    constants::{AUTH_SEED, POOL_SEED},
    curve::{creator_fee, fund_fee, protocol_fee, swap_base_output_without_fees, trade_fee},
    error::ErrorCode,
    states::{PoolState, PoolStatusBitIndex},
    AmmConfig,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
#[derive(Accounts)]
pub struct SwapBaseOutput<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut, seeds=[POOL_SEED, pool_state.amm_config.key().as_ref(), pool_state.token_0_mint.key().as_ref(), pool_state.token_1_mint.key().as_ref()], bump = pool_state.bump)]
    pub pool_state: Box<Account<'info, PoolState>>,
    #[account(address = pool_state.amm_config)]
    pub amm_config: Box<Account<'info, AmmConfig>>,
    #[account(address = input_vault.mint @ ErrorCode::MintMismatch)]
    pub input_token_mint: Account<'info, Mint>,
    #[account(address = output_vault.mint @ ErrorCode::MintMismatch)]
    pub output_token_mint: Account<'info, Mint>,
    #[account(mut, constraint = input_vault.key() == pool_state.token_0_vault || input_vault.key() == pool_state.token_1_vault @ ErrorCode::InvalidVault)]
    pub input_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = output_vault.key() == pool_state.token_0_vault || output_vault.key() == pool_state.token_1_vault @ ErrorCode::InvalidVault, constraint = output_vault.key() != input_vault.key() @ ErrorCode::SameVault)]
    pub output_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = input_token_mint, token::authority = signer)]
    pub input_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = output_token_mint, token::authority = signer)]
    pub output_token_account: Box<Account<'info, TokenAccount>>,
    /// CHECK: Authority PDA
    #[account(seeds=[AUTH_SEED], bump = pool_state.auth_bump)]
    pub authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}
pub fn swap_base_output(
    ctx: Context<SwapBaseOutput>,
    amount_out: u64,        // Exact output user wants
    maximum_amount_in: u64, // Max user is willing to pay (slippage)
) -> Result<()> {
    let pool_state = &mut ctx.accounts.pool_state;
    require!(
        pool_state.is_enabled(PoolStatusBitIndex::Swap),
        ErrorCode::SwapBlocked
    );
    require!(
        pool_state.open_time < Clock::get()?.unix_timestamp as u64,
        ErrorCode::SwapBlocked
    );
    require!(amount_out > 0, ErrorCode::InvalidTokenAmount);
    require!(maximum_amount_in > 0, ErrorCode::InvalidTokenAmount);
    // Step 1: Determine swap direction
    let is_token_0_input = ctx.accounts.input_vault.key() == pool_state.token_0_vault;
    // Step 2: Get vault amounts in correct order
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
    // Step 4: Map to input/output
    let (input_vault_balance, output_vault_balance) = if is_token_0_input {
        (clean_vault_0 as u128, clean_vault_1 as u128)
    } else {
        (clean_vault_1 as u128, clean_vault_0 as u128)
    };
    // k verification: constant BEFORE
    let constant_before = input_vault_balance
        .checked_mul(output_vault_balance)
        .ok_or(ErrorCode::MathOverflow)?;
    // Calculate required input for desired output (BEFORE fees)
    let input_without_fee = swap_base_output_without_fees(
        amount_out as u128,
        input_vault_balance,
        output_vault_balance,
    )
    .ok_or(ErrorCode::MathOverflow)?;
    // Add trade fee to get total input needed
    // input_with_fee = input_without_fee / (1 - fee_rate)
    // Simplified: input_with_fee = input_without_fee * denominator / (denominator - fee_rate)
    let fee_denominator = 1_000_000u128;
    let fee_rate = ctx.accounts.amm_config.trade_fee_rate as u128;
    let amount_in = input_without_fee
        .checked_mul(fee_denominator)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(
            fee_denominator
                .checked_sub(fee_rate)
                .ok_or(ErrorCode::MathOverflow)?,
        )
        .ok_or(ErrorCode::MathOverflow)?;
    // Calculate fee from amount_in
    let fee = trade_fee(amount_in, ctx.accounts.amm_config.trade_fee_rate)
        .ok_or(ErrorCode::MathOverflow)?;
    // Slippage check: input must not exceed maximum
    require!(
        amount_in as u64 <= maximum_amount_in,
        ErrorCode::SlippageExceeded
    );
    // Split fees
    let protocol_fee_amount = protocol_fee(fee, ctx.accounts.amm_config.protocol_fee_rate)
        .ok_or(ErrorCode::MathOverflow)?;
    let fund_fee_amount =
        fund_fee(fee, ctx.accounts.amm_config.fund_fee_rate).ok_or(ErrorCode::MathOverflow)?;
    let creator_fee_amount = creator_fee(fee, ctx.accounts.amm_config.creator_fee_rate)
        .ok_or(ErrorCode::MathOverflow)?;
    // Track fees
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
    // k verification: constant AFTER
    let actual_input = amount_in.checked_sub(fee).ok_or(ErrorCode::MathOverflow)?;
    let new_input_balance = input_vault_balance
        .checked_add(actual_input)
        .ok_or(ErrorCode::MathOverflow)?;
    let new_output_balance = output_vault_balance
        .checked_sub(amount_out as u128)
        .ok_or(ErrorCode::MathOverflow)?;
    let constant_after = new_input_balance
        .checked_mul(new_output_balance)
        .ok_or(ErrorCode::MathOverflow)?;
    require!(
        constant_after >= constant_before,
        ErrorCode::ConstantProductInvariant
    );
    // Transfer IN
    let transfer_in = Transfer {
        from: ctx.accounts.input_token_account.to_account_info(),
        to: ctx.accounts.input_vault.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };
    token::transfer(
        CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_in),
        amount_in as u64,
    )?;
    // Transfer OUT
    let seeds = &[AUTH_SEED, &[pool_state.auth_bump]];
    let signer_seeds = &[&seeds[..]];
    let transfer_out = Transfer {
        from: ctx.accounts.output_vault.to_account_info(),
        to: ctx.accounts.output_token_account.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_out,
            signer_seeds,
        ),
        amount_out,
    )?;
    Ok(())
}
