pub const FEE_RATE_DENOMINATOR: u64 = 1_000_000;
fn floor_div(amount: u128, rate: u64) -> Option<u128> {
    let value = (amount.checked_mul(rate as u128)?).checked_div(FEE_RATE_DENOMINATOR as u128)?;
    Some(value)
}
fn ceil_div(amount: u128, rate: u64) -> Option<u128> {
    let x = amount.checked_mul(rate as u128)?;
    let y = FEE_RATE_DENOMINATOR as u128;
    let num = x.checked_add(y.checked_sub(1)?)?;
    let den = y;
    let value = num.checked_div(den)?;
    Some(value)
}
/// Trade fee — rounds UP (user pays more)
pub fn trade_fee(amount: u128, rate: u64) -> Option<u128> {
    ceil_div(amount, rate)
}
/// Protocol fee from trade fee — rounds DOWN
pub fn protocol_fee(trade_fee: u128, rate: u64) -> Option<u128> {
    floor_div(trade_fee, rate)
}
/// Fund fee from trade fee — rounds DOWN
pub fn fund_fee(trade_fee: u128, rate: u64) -> Option<u128> {
    floor_div(trade_fee, rate)
}
/// Creator fee — rounds UP
pub fn creator_fee(amount: u128, rate: u64) -> Option<u128> {
    ceil_div(amount, rate)
}