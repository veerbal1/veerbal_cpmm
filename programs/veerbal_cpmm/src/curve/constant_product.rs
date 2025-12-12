pub fn swap_base_input_without_fees(
    input_amount: u128,
    input_vault_amount: u128,
    output_vault_amount: u128,
) -> Option<u128> {
    let num = (input_amount as u128).checked_mul(output_vault_amount)?;
    let den = input_vault_amount.checked_add(input_amount as u128)?;
    let output_amount = num.checked_div(den)?;

    Some(output_amount)
}

pub fn swap_base_output_without_fees(
    output_amount: u128,
    input_vault_amount: u128,
    output_vault_amount: u128,
) -> Option<u128> {
    if output_amount >= output_vault_amount {
        return None;
    }

    let numerator = input_vault_amount.checked_mul(output_amount)?;
    let denominator = output_vault_amount.checked_sub(output_amount)?;

    let input_amount =
        (numerator.checked_add(denominator.checked_sub(1)?)?).checked_div(denominator)?;

    Some(input_amount)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_basic_swap() {
        // Pool: 1000/1000, swap 100 in
        println!("[[[Testing Swap Functionality]]]]]");
        // Expected: (100 * 1000) / (1000 + 100) = 90
        assert_eq!(swap_base_input_without_fees(100, 1000, 1000), Some(90));
    }
}
