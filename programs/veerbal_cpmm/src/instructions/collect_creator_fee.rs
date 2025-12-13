use crate::{
    constants::{AUTH_SEED, POOL_SEED, VAULT_SEED},
    error::ErrorCode,
    states::PoolState
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
    pub system_program: Program<'info, System>,
}

pub fn collect_creator_fee(ctx: Context<CollectCreatorFee>) -> Result<()> {
    let pool_state = &mut ctx.accounts.pool_state;

    let fee_0 = pool_state.creator_token_0_fee;
    let fee_1 = pool_state.creator_token_1_fee;

    require!(fee_0 > 0 || fee_1 > 0, ErrorCode::CreatorFeeNotAccumulated);

    let seeds = &[AUTH_SEED, &[pool_state.auth_bump]];
    let signer_seeds = &[&seeds[..]];

    if fee_0 > 0 {
        let accounts = Transfer {
            from: ctx.accounts.token_0_vault.to_account_info(),
            to: ctx.accounts.receiver_token_0_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };

        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            accounts,
            signer_seeds,
        );

        token::transfer(cpi_context, fee_0)?
    }

    // 5. Transfer token_1 fees (if any)
    if fee_1 > 0 {
        let accounts = Transfer {
            from: ctx.accounts.token_1_vault.to_account_info(),
            to: ctx.accounts.receiver_token_1_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };

        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            accounts,
            signer_seeds,
        );

        token::transfer(cpi_context, fee_1)?
    }

    // 6. Reset fee counters
    pool_state.creator_token_0_fee = 0;
    pool_state.creator_token_1_fee = 0;

    // 7. Update recent_epoch
    pool_state.recent_epoch = Clock::get()?.epoch;
    Ok(())
}
