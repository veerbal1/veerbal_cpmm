use crate::{
    constants::{AUTH_SEED, LP_MINT_SEED, POOL_SEED},
    error::ErrorCode,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

use crate::{
    instructions::CONFIG_SEED,
    states::{PoolState, PoolStatusBitIndex},
    AmmConfig,
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
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

    #[account(mut, token::mint = lp_mint, token::authority = signer)]
    pub signer_lp: Account<'info, TokenAccount>,

    #[account(mut, seeds=[LP_MINT_SEED, pool_state.key().as_ref()], bump = pool_state.mint_bump)]
    pub lp_mint: Account<'info, Mint>,

    // pool state dependent upon amm config
    #[account(seeds=[CONFIG_SEED, amm_config.index.to_be_bytes().as_ref()], bump = amm_config.bump)]
    pub amm_config: Account<'info, AmmConfig>,

    // Authority pda is needed to sign vault transactions
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(seeds=[AUTH_SEED], bump = pool_state.auth_bump)]
    pub authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn withdraw(
    ctx: Context<Withdraw>,
    lp_amount: u64,
    minimum_token_0_amount: u64,
    minimum_token_1_amount: u64,
) -> Result<()> {
    let pool_state = &mut ctx.accounts.pool_state;

    // 1.  VALIDATION
    require!(
        pool_state.is_enabled(PoolStatusBitIndex::Withdraw),
        ErrorCode::WithdrawDisabled
    );

    require!(lp_amount > 0, ErrorCode::InvalidLPAmount);

    require!(pool_state.lp_supply > 0, ErrorCode::PoolNotInitialized);

    // 2.  CALCULATE TOKEN AMOUNTS (round DOWN!)
    // CALCULATE TOKEN AMOUNTS (round DOWN!)
    let (clean_vault_0, clean_vault_1) = pool_state.vault_amount_without_fee(
        ctx.accounts.token_0_vault.amount,
        ctx.accounts.token_1_vault.amount,
    )?;

    let token_0_amount = (((lp_amount as u128)
        .checked_mul(clean_vault_0 as u128)
        .ok_or(ErrorCode::MathOverflow)?)
    .checked_div(pool_state.lp_supply as u128)
    .ok_or(ErrorCode::MathOverflow)?)
    .try_into()
    .map_err(|_| ErrorCode::MathOverflow)?;

    let token_1_amount = (((lp_amount as u128)
        .checked_mul(clean_vault_1 as u128)
        .ok_or(ErrorCode::MathOverflow)?)
    .checked_div(pool_state.lp_supply as u128)
    .ok_or(ErrorCode::MathOverflow)?)
    .try_into()
    .map_err(|_| ErrorCode::MathOverflow)?;

    require!(token_0_amount > 0, ErrorCode::ZeroTradingTokens);
    require!(token_1_amount > 0, ErrorCode::ZeroTradingTokens);

    // 3. SLIPPAGE CHECK
    require!(
        token_0_amount >= minimum_token_0_amount,
        ErrorCode::SlippageExceeded
    );

    require!(
        token_1_amount >= minimum_token_1_amount,
        ErrorCode::SlippageExceeded
    );

    // 4.  EFFECTS FIRST (state changes before external calls!)
    pool_state.lp_supply = pool_state
        .lp_supply
        .checked_sub(lp_amount)
        .ok_or(ErrorCode::MathOverflow)?;

    let cpi_accounts = Burn {
        mint: ctx.accounts.lp_mint.to_account_info(),
        from: ctx.accounts.signer_lp.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };

    let cpi_context = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);

    token::burn(cpi_context, lp_amount)?;

    // 5.  INTERACTIONS LAST
    let cpi_accounts = Transfer {
        from: ctx.accounts.token_0_vault.to_account_info(),
        to: ctx.accounts.signer_token_0.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };

    let seeds = &[AUTH_SEED, &[pool_state.auth_bump]];
    let signer_seeds = &[&seeds[..]];

    let token_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(token_program, cpi_accounts, signer_seeds);
    token::transfer(cpi_ctx, token_0_amount)?;

    let cpi_accounts = Transfer {
        from: ctx.accounts.token_1_vault.to_account_info(),
        to: ctx.accounts.signer_token_1.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };

    let token_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(token_program, cpi_accounts, signer_seeds);
    token::transfer(cpi_ctx, token_1_amount)?;

    Ok(())
}
