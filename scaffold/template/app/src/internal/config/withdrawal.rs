use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct WithdrawalFeeConfig {
    /// Fee percentage (0.0 = no fee, 0.02 = 2%)
    pub fee_percentage: Decimal,
    /// Fixed fee amount added on top of percentage
    pub fee_fixed: Decimal,
    /// Minimum withdrawal amount
    pub min_amount: Decimal,
}

impl Default for WithdrawalFeeConfig {
    fn default() -> Self {
        Self {
            fee_percentage: Decimal::ZERO,
            fee_fixed: Decimal::ZERO,
            min_amount: Decimal::ZERO,
        }
    }
}

impl WithdrawalFeeConfig {
    /// Calculate the fee and net amount for a given withdrawal amount.
    /// Returns (fee, net_amount).
    pub fn calculate_fee(&self, amount: Decimal) -> (Decimal, Decimal) {
        let fee = (amount * self.fee_percentage) + self.fee_fixed;
        let net_amount = amount - fee;
        (fee, net_amount)
    }
}
