use anchor_lang::prelude::*;
pub use states::AmmConfig;

pub mod states;
pub mod instructions;
pub mod error;
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

fn calculate_deposit_amounts(
    lp_tokens_to_mint: u64,
    lp_supply: u64,
    vault_0_amount: u64,
    vault_1_amount: u64,
) -> (u64, u64) {
    // Calculate the share of the pool the user is buying.
    // Use u128 for intermediate multiplication to prevent overflow before division.
    // This implicitly rounds down, which favors the pool by requiring slightly more tokens for the same LP amount.

    let token_0_amount: u64 = ((vault_0_amount as u128)
        .checked_mul(lp_tokens_to_mint as u128)
        .unwrap())
        .checked_div(lp_supply as u128)
        .unwrap() as u64;
    let token_1_amount: u64 = ((vault_1_amount as u128)
        .checked_mul(lp_tokens_to_mint as u128)
        .unwrap())
        .checked_div(lp_supply as u128)
        .unwrap() as u64;

    (token_0_amount, token_1_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_deposit_amounts_basic() {
        // Simulate a pool with 1000 LP supply, 10000 tokens in each vault
        let lp_tokens_to_mint = 50;
        let lp_supply = 500;
        let vault_0_amount = 1000;
        let vault_1_amount = 1000; // Adjusted to get 100 for amount_1 as well

        let (amount_0, amount_1) =
            calculate_deposit_amounts(lp_tokens_to_mint, lp_supply, vault_0_amount, vault_1_amount);

        // Basic proportional check: user gets 10% of the vault tokens
        assert_eq!(amount_0, 100);
        assert_eq!(amount_1, 100);
    }
}
