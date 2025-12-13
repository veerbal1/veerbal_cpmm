use crate::instructions::*;
use crate::states::*;
use anchor_lang::prelude::*;

pub mod constants;
pub mod curve;
pub mod error;
pub mod instructions;
pub mod states;

declare_id!("C6TCz92bpYjWgty9mwrAoNh7u6RSdmyBRB4dMoBGgMrA");
pub const ADMIN: Pubkey = pubkey!("C6TCz92bpYjWgty9mwrAoNh7u6RSdmyBRB4dMoBGgMrA");

#[program]
pub mod veerbal_cpmm {
    use super::*;

    pub fn create_config(
        ctx: Context<CreateAmmConfig>,
        index: u16,
        trade_fee_rate: u64,
        creator_fee_rate: u64,
        protocol_fee_rate: u64,
        fund_fee_rate: u64,
        create_pool_fee: u64,
    ) -> Result<()> {
        instructions::create_amm_config(
            ctx,
            index,
            trade_fee_rate,
            creator_fee_rate,
            protocol_fee_rate,
            fund_fee_rate,
            create_pool_fee,
        )?;
        Ok(())
    }

    pub fn create_pool(
        ctx: Context<CreatePool>,
        index: u16,
        init_amount_0: u64,
        init_amount_1: u64,
        open_time: u64,
    ) -> Result<()> {
        instructions::create_pool(ctx, index, init_amount_0, init_amount_1, open_time)?;
        Ok(())
    }

    pub fn deposit(
        ctx: Context<Deposit>,
        lp_amount: u64,
        maximum_token_0_amount: u64,
        maximum_token_1_amount: u64,
    ) -> Result<()> {
        instructions::deposit(
            ctx,
            lp_amount,
            maximum_token_0_amount,
            maximum_token_1_amount,
        )?;
        Ok(())
    }

    pub fn withdraw(
        ctx: Context<Withdraw>,
        lp_amount: u64,
        minimum_token_0_amount: u64,
        minimum_token_1_amount: u64,
    ) -> Result<()> {
        instructions::withdraw(
            ctx,
            lp_amount,
            minimum_token_0_amount,
            minimum_token_1_amount,
        )?;
        Ok(())
    }

    pub fn swap(ctx: Context<Swap>, amount_in: u64, minimum_amount_out: u64) -> Result<()> {
        instructions::swap(ctx, amount_in, minimum_amount_out)?;
        Ok(())
    }

    pub fn swap_base_output(
        ctx: Context<SwapBaseOutput>,
        amount_out: u64,
        maximum_amount_in: u64,
    ) -> Result<()> {
        instructions::swap_base_output(ctx, amount_out, maximum_amount_in)?;
        Ok(())
    }

    pub fn collect_creator_fee(ctx: Context<CollectCreatorFee>) -> Result<()> {
        instructions::collect_creator_fee(ctx)
    }
    pub fn collect_protocol_fee(ctx: Context<CollectProtocolFee>) -> Result<()> {
        instructions::collect_protocol_fee(ctx)
    }
    pub fn collect_fund_fee(ctx: Context<CollectFundFee>) -> Result<()> {
        instructions::collect_fund_fee(ctx)
    }
}
