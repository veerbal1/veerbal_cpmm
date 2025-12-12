use anchor_lang::prelude::*;
pub use states::AmmConfig;

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

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
