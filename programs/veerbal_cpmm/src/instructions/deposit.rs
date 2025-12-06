use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Token, TokenAccount, Transfer},
};

use crate::{
    constants::{AUTH_SEED, LP_MINT_SEED, POOL_SEED},
    error::ErrorCode,
    instructions::CONFIG_SEED,
    states::{PoolState, PoolStatusBitIndex},
    AmmConfig,
};

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

pub fn deposit(
    ctx: Context<Deposit>,
    lp_amount: u64,
    maximum_token_0_amount: u64,
    maximum_token_1_amount: u64,
) -> Result<()> {
    let pool_state = &mut ctx.accounts.pool_state;

    require!(
        pool_state.is_enabled(PoolStatusBitIndex::Deposit),
        ErrorCode::DepositDisabled
    );

    require!(lp_amount > 0, ErrorCode::InvalidLPAmount);
    require!(maximum_token_0_amount > 0, ErrorCode::InvalidTokenAmount);
    require!(maximum_token_1_amount > 0, ErrorCode::InvalidTokenAmount);

    // Pool must be initialized (lp_supply > 0 from initialize)
    require!(pool_state.lp_supply > 0, ErrorCode::PoolNotInitialized);

    // TODO: Step 2 — Calculate required tokens for requested lp_amount
    let lp_supply = pool_state.lp_supply;
    let token_0_vault = &ctx.accounts.token_0_vault;
    let token_1_vault = &ctx.accounts.token_1_vault;
    let num = (lp_amount as u128)
        .checked_mul(token_0_vault.amount as u128)
        .ok_or(ErrorCode::MathOverflow)?;
    let token_0_amount = num
        .div_ceil(lp_supply as u128)
        .try_into()
        .map_err(|_| ErrorCode::MathOverflow)?;

    let num = (lp_amount as u128)
        .checked_mul(token_1_vault.amount as u128)
        .ok_or(ErrorCode::MathOverflow)?;
    let token_1_amount = num
        .div_ceil(lp_supply as u128)
        .try_into()
        .map_err(|_| ErrorCode::MathOverflow)?;

    require!(
        token_0_amount <= maximum_token_0_amount,
        ErrorCode::MaximumAmountExceed
    );
    require!(
        token_1_amount <= maximum_token_1_amount,
        ErrorCode::MaximumAmountExceed
    );

    // TODO: Step 3 — Transfer tokens from user → vaults
    let accounts_0 = Transfer {
        from: ctx.accounts.signer_token_0.to_account_info(),
        to: ctx.accounts.token_0_vault.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };

    let token_program = ctx.accounts.token_program.to_account_info();
    let cpi = CpiContext::new(token_program, accounts_0);
    token::transfer(cpi, token_0_amount)?;

    let accounts_1 = Transfer {
        from: ctx.accounts.signer_token_1.to_account_info(),
        to: ctx.accounts.token_1_vault.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };

    let token_program = ctx.accounts.token_program.to_account_info();
    let cpi = CpiContext::new(token_program, accounts_1);
    token::transfer(cpi, token_1_amount)?;

    // TODO: Step 4 — Mint LP tokens to user
    let mint_accounts = MintTo {
        mint: ctx.accounts.lp_mint.to_account_info(),
        to: ctx.accounts.signer_lp.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };

    let seeds = &[AUTH_SEED, &[pool_state.auth_bump]];
    let signer_seeds = &[&seeds[..]];
    let token_program = ctx.accounts.token_program.to_account_info();

    let cpi_ctx = CpiContext::new_with_signer(token_program, mint_accounts, signer_seeds);
    token::mint_to(cpi_ctx, lp_amount)?;

    // TODO: Step 5 — Update lp_supply
    pool_state.lp_supply = pool_state
        .lp_supply
        .checked_add(lp_amount)
        .ok_or(ErrorCode::MathOverflow)?;

    Ok(())
}
