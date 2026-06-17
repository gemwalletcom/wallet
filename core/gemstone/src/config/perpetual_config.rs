use primitives::asset_constants::ARBITRUM_USDC_ASSET_ID;

pub const DEFAULT_LEVERAGE: u8 = 5;
pub const LEVERAGE_OPTIONS: &[u8] = &[1, 2, 3, 5, 10, 20, 25, 30, 40, 50];

pub const HYPERLIQUID_DEPOSIT_ADDRESS: &str = "0x2Df1c51E09aECF9cacB7bc98cB1742757f163dF7";

pub const MIN_DEPOSIT_AMOUNT: u64 = 5_000_000;
pub const MIN_WITHDRAW_AMOUNT: u64 = 2_000_000;

pub const PRICES_UPDATE_INTERVAL_SECONDS: u32 = 5;

pub const TAKE_PROFIT_PERCENT_OPTIONS: &[u8] = &[0, 10, 25, 50, 100, 200];
pub const STOP_LOSS_PERCENT_OPTIONS: &[u8] = &[0, 3, 5, 10, 25, 50];
pub const DEFAULT_TAKE_PROFIT_PERCENT: u8 = 0;
pub const DEFAULT_STOP_LOSS_PERCENT: u8 = 0;
const AUTOCLOSE_SUGGESTION_TIERS: &[(u8, &[u8])] = &[(3, &[5, 10, 15]), (5, &[10, 15, 25]), (10, &[15, 25, 50])];
const AUTOCLOSE_SUGGESTIONS_HIGH_LEVERAGE: &[u8] = &[25, 50, 100];

#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct PerpetualConfig {
    pub default_leverage: u8,
    pub leverage_options: Vec<u8>,
    pub take_profit_percent_options: Vec<u8>,
    pub stop_loss_percent_options: Vec<u8>,
    pub default_take_profit_percent: u8,
    pub default_stop_loss_percent: u8,
    pub deposit_address: String,
    pub deposit_asset_id: String,
    pub min_deposit: u64,
    pub min_withdraw: u64,
    pub prices_update_interval_seconds: u32,
}

pub fn get_perpetual_config() -> PerpetualConfig {
    PerpetualConfig {
        default_leverage: DEFAULT_LEVERAGE,
        leverage_options: LEVERAGE_OPTIONS.to_vec(),
        take_profit_percent_options: TAKE_PROFIT_PERCENT_OPTIONS.to_vec(),
        stop_loss_percent_options: STOP_LOSS_PERCENT_OPTIONS.to_vec(),
        default_take_profit_percent: DEFAULT_TAKE_PROFIT_PERCENT,
        default_stop_loss_percent: DEFAULT_STOP_LOSS_PERCENT,
        deposit_address: HYPERLIQUID_DEPOSIT_ADDRESS.to_string(),
        deposit_asset_id: ARBITRUM_USDC_ASSET_ID.to_string(),
        min_deposit: MIN_DEPOSIT_AMOUNT,
        min_withdraw: MIN_WITHDRAW_AMOUNT,
        prices_update_interval_seconds: PRICES_UPDATE_INTERVAL_SECONDS,
    }
}

pub fn select_leverage(desired: u8, options: &[u8]) -> u8 {
    options
        .iter()
        .copied()
        .filter(|&value| value <= desired)
        .max()
        .or_else(|| options.iter().copied().min())
        .unwrap_or(DEFAULT_LEVERAGE)
}

pub fn get_autoclose_suggestions(leverage: u8) -> Vec<u8> {
    AUTOCLOSE_SUGGESTION_TIERS
        .iter()
        .find(|&&(max_leverage, _)| leverage <= max_leverage)
        .map_or(AUTOCLOSE_SUGGESTIONS_HIGH_LEVERAGE, |&(_, suggestions)| suggestions)
        .to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_leverage() {
        assert_eq!(select_leverage(0, LEVERAGE_OPTIONS), 1);
        assert_eq!(select_leverage(4, LEVERAGE_OPTIONS), 3);
        assert_eq!(select_leverage(5, LEVERAGE_OPTIONS), 5);
        assert_eq!(select_leverage(7, LEVERAGE_OPTIONS), 5);
        assert_eq!(select_leverage(50, LEVERAGE_OPTIONS), 50);
        assert_eq!(select_leverage(100, LEVERAGE_OPTIONS), 50);

        let constrained: &[u8] = &[1, 2, 3];
        assert_eq!(select_leverage(10, constrained), 3);

        let empty: &[u8] = &[];
        assert_eq!(select_leverage(5, empty), DEFAULT_LEVERAGE);
    }

    #[test]
    fn test_get_perpetual_config() {
        let config = get_perpetual_config();
        assert_eq!(config.default_leverage, DEFAULT_LEVERAGE);
        assert_eq!(config.leverage_options, LEVERAGE_OPTIONS);
        assert_eq!(config.take_profit_percent_options, TAKE_PROFIT_PERCENT_OPTIONS);
        assert_eq!(config.stop_loss_percent_options, STOP_LOSS_PERCENT_OPTIONS);
        assert_eq!(config.default_take_profit_percent, DEFAULT_TAKE_PROFIT_PERCENT);
        assert_eq!(config.default_stop_loss_percent, DEFAULT_STOP_LOSS_PERCENT);
        assert_eq!(config.deposit_address, HYPERLIQUID_DEPOSIT_ADDRESS);
        assert_eq!(config.deposit_asset_id, ARBITRUM_USDC_ASSET_ID.to_string());
        assert_eq!(config.min_deposit, MIN_DEPOSIT_AMOUNT);
        assert_eq!(config.min_withdraw, MIN_WITHDRAW_AMOUNT);
        assert_eq!(config.prices_update_interval_seconds, PRICES_UPDATE_INTERVAL_SECONDS);
    }

    #[test]
    fn test_get_autoclose_suggestions() {
        assert_eq!(get_autoclose_suggestions(2), vec![5, 10, 15]);
        assert_eq!(get_autoclose_suggestions(5), vec![10, 15, 25]);
        assert_eq!(get_autoclose_suggestions(10), vec![15, 25, 50]);
        assert_eq!(get_autoclose_suggestions(25), vec![25, 50, 100]);
    }
}
