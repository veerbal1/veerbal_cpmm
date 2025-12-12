use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Token, TokenAccount, Transfer},
};

use crate::{
    constants::{AUTH_SEED, LP_MINT_SEED, POOL_SEED, VAULT_SEED},
    error::ErrorCode,
    instructions::CONFIG_SEED,
    states::{pool, PoolState},
    AmmConfig,
};

#[derive(Accounts)]
#[instruction(index: u16)]
pub struct Initialize<'info> {
    // 1. Who pays and signs?
    #[account(mut)]
    pub creator: Signer<'info>,

    // 2. Which config does this pool use?
    #[account(seeds=[CONFIG_SEED,index.to_be_bytes().as_ref()], bump = amm_config.bump)]
    pub amm_config: Account<'info, AmmConfig>,

    // 3. The pool state we're creating (what seeds?)
    #[account(
        init, 
        seeds=[POOL_SEED, amm_config.key().as_ref(), token_0_mint.key().as_ref(), token_1_mint.key().as_ref()], 
        bump, 
        payer = creator, 
        space=PoolState::LEN
    )]
    pub pool_state: Account<'info, PoolState>,

    #[account(constraint = token_0_mint.key() < token_1_mint.key())]
    pub token_0_mint: Account<'info, Mint>,
    pub token_1_mint: Account<'info, Mint>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(seeds = [AUTH_SEED], bump)]
    pub authority: UncheckedAccount<'info>,

    #[account(init, seeds=[LP_MINT_SEED, pool_state.key().as_ref()], bump, payer = creator, mint::decimals = 9, mint::authority = authority)]
    pub lp_mint: Account<'info, Mint>,

    #[account(init, seeds=[VAULT_SEED, pool_state.key().as_ref(), token_0_mint.key().as_ref()], bump, payer = creator, token::mint = token_0_mint, token::authority = authority)]
    pub token_0_vault: Account<'info, TokenAccount>,

    #[account(init, seeds=[VAULT_SEED, pool_state.key().as_ref(), token_1_mint.key().as_ref()], bump, payer = creator, token::mint = token_1_mint, token::authority = authority)]
    pub token_1_vault: Account<'info, TokenAccount>,

    // Creator
    #[account(mut, token::mint = token_0_mint, token::authority = creator)]
    pub creator_token_0: Account<'info, TokenAccount>,

    #[account(mut, token::mint = token_1_mint, token::authority = creator)]
    pub creator_token_1: Account<'info, TokenAccount>,

    #[account(init, associated_token::mint = lp_mint, associated_token::authority = creator, payer = creator)]
    pub creator_lp: Account<'info, TokenAccount>,

    /// CHECK: Normal Sol Wallet Account
    #[account(mut, address = amm_config.fund_owner @ ErrorCode::InvalidFeeReceiver)]
    pub fee_receiver: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn initialize(
    ctx: Context<Initialize>,
    _index: u16,
    init_amount_0: u64,
    init_amount_1: u64,
    open_time: u64,
) -> Result<()> {

    require!(init_amount_0 > 0, ErrorCode::InvalidTokenAmount);
    require!(init_amount_1 > 0, ErrorCode::InvalidTokenAmount);

    let amm_config = &ctx.accounts.amm_config;
    require!(
        !amm_config.disable_create_pool,
        ErrorCode::PoolCreationDisabled
    );

    let token_program_token_0 = ctx.accounts.token_program.to_account_info();

    let cpi_transfer_token_0_accounts = Transfer {
        from: ctx.accounts.creator_token_0.to_account_info(),
        to: ctx.accounts.token_0_vault.to_account_info(),
        authority: ctx.accounts.creator.to_account_info(),
    };

    let cpi_context = CpiContext::new(token_program_token_0, cpi_transfer_token_0_accounts);
    token::transfer(cpi_context, init_amount_0)?;

    // Step 3
    let cpi_transfer_token_1_accounts = Transfer {
        from: ctx.accounts.creator_token_1.to_account_info(),
        to: ctx.accounts.token_1_vault.to_account_info(),
        authority: ctx.accounts.creator.to_account_info(),
    };

    let token_program_token_1 = ctx.accounts.token_program.to_account_info();
    let cpi_context = CpiContext::new(token_program_token_1, cpi_transfer_token_1_accounts);
    token::transfer(cpi_context, init_amount_1)?;

    // Just calculate intial LP tokens to mint
    let product = (init_amount_0 as u128) * (init_amount_1 as u128);
    let intitial_lp_liquidity = product.isqrt() as u64;

    require!(
        intitial_lp_liquidity > 100,
        ErrorCode::InsufficientTokensToMint
    );

    let creator_lp_amount = intitial_lp_liquidity - 100;

    let seeds = &[AUTH_SEED, &[ctx.bumps.authority]];
    let signer_seeds = &[&seeds[..]];

    let mint_accounts = MintTo {
        mint: ctx.accounts.lp_mint.to_account_info(),
        to: ctx.accounts.creator_lp.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };

    let mint_program = ctx.accounts.token_program.to_account_info();
    let mint_context = CpiContext::new_with_signer(mint_program, mint_accounts, signer_seeds);
    token::mint_to(mint_context, creator_lp_amount)?;

    // Transfer pool creation fee.
    let amount = amm_config.create_pool_fee;
    let cpi_ctx = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
            from: ctx.accounts.creator.to_account_info(),
            to: ctx.accounts.fee_receiver.to_account_info(),
        },
    );
    system_program::transfer(cpi_ctx, amount)?;

    // Step 8: Initialize pool_state with all fields
    let pool_state = &mut ctx.accounts.pool_state;
    pool_state.amm_config = amm_config.key();
    pool_state.pool_creator = ctx.accounts.creator.key();
    pool_state.token_0_vault = ctx.accounts.token_0_vault.key();
    pool_state.token_1_vault = ctx.accounts.token_1_vault.key();
    pool_state.token_0_mint = ctx.accounts.token_0_mint.key();
    pool_state.token_1_mint = ctx.accounts.token_1_mint.key();
    pool_state.lp_mint = ctx.accounts.lp_mint.key();
    pool_state.token_0_program = ctx.accounts.token_program.key();
    pool_state.token_1_program = ctx.accounts.token_program.key();

    // Config
    pool_state.auth_bump = ctx.bumps.authority;
    pool_state.bump = ctx.bumps.pool_state;
    pool_state.token_0_bump = ctx.bumps.token_0_vault;
    pool_state.token_1_bump = ctx.bumps.token_1_vault;
    pool_state.mint_bump = ctx.bumps.lp_mint;

    pool_state.status = 0; // all operations enabled
    pool_state.open_time = open_time;
    pool_state.recent_epoch = Clock::get()?.epoch;

    pool_state.lp_supply = intitial_lp_liquidity;

    pool_state.mint_0_decimals = ctx.accounts.token_0_mint.decimals;
    pool_state.mint_1_decimals = ctx.accounts.token_1_mint.decimals;
    pool_state.lp_mint_decimals = 9;

    pool_state.protocol_token_0_fee = 0;
    pool_state.protocol_token_1_fee = 0;
    pool_state.fund_token_0_fee = 0;
    pool_state.fund_token_1_fee = 0;
    pool_state.creator_token_0_fee = 0;
    pool_state.creator_token_1_fee = 0;

    pool_state.creator_fee_on = 0;
    pool_state.creator_fee_active = true;
    Ok(())
}
