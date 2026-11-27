mod language;

use language::{Arguments, FALLBACK, id};

pub struct LanguageLocalizer {
    language: &'static str,
}

pub struct LanguageNotification {
    pub title: String,
    pub description: String,
}

pub enum TransactionAction {
    Transfer,
    Swap,
    TokenApproval,
    Stake,
    Unstake,
    Redelegate,
    ClaimRewards,
    Withdraw,
    Freeze,
    Unfreeze,
    SmartContract,
}

impl Default for LanguageLocalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageLocalizer {
    pub fn new() -> Self {
        Self::new_with_language(FALLBACK)
    }

    pub fn new_with_language(language: &str) -> Self {
        Self { language: language::name(language) }
    }

    fn format(&self, id: &str, arguments: &Arguments) -> String {
        language::format(self.language, id, arguments)
    }

    pub fn price_alert_up(&self, symbol: &str, price: &str, price_change: &str) -> LanguageNotification {
        LanguageNotification {
            title: self.format(id::notification_price_alert_up_title, &[("symbol", &symbol)]),
            description: self.format(id::notification_price_alert_up_description, &[("price", &price), ("price_change", &price_change)]),
        }
    }

    pub fn price_alert_down(&self, symbol: &str, price: &str, price_change: &str) -> LanguageNotification {
        LanguageNotification {
            title: self.format(id::notification_price_alert_down_title, &[("symbol", &symbol)]),
            description: self.format(id::notification_price_alert_down_description, &[("price", &price), ("price_change", &price_change)]),
        }
    }

    pub fn price_alert_target(&self, symbol: &str, target_price: &str, current_price: &str, change: &str) -> LanguageNotification {
        LanguageNotification {
            title: self.format(id::notification_price_alert_target_title, &[("symbol", &symbol), ("price", &target_price)]),
            description: self.format(id::notification_price_alert_target_description, &[("price", &current_price), ("change", &change)]),
        }
    }

    pub fn price_alert_all_time_high(&self, symbol: &str, price: &str) -> LanguageNotification {
        LanguageNotification {
            title: self.format(id::notification_price_alert_all_time_high_title, &[("symbol", &symbol)]),
            description: self.format(id::notification_price_alert_all_time_high_description, &[("symbol", &symbol), ("price", &price)]),
        }
    }

    pub fn notification_transfer_title(&self, is_sent: bool, value: &str) -> String {
        if is_sent {
            self.format(id::notification_sent_title, &[("value", &value)])
        } else {
            self.format(id::notification_received_title, &[("value", &value)])
        }
    }

    pub fn notification_nft_transfer_title(&self, is_sent: bool, value: &str) -> String {
        if is_sent {
            self.format(id::notification_nft_sent_title, &[("value", &value)])
        } else {
            self.format(id::notification_nft_received_title, &[("value", &value)])
        }
    }

    pub fn notification_transfer_description(&self, is_sent: bool, to_address: &str, from_address: &str) -> String {
        if is_sent {
            self.format(id::notification_sent_description, &[("address", &to_address)])
        } else {
            self.format(id::notification_received_description, &[("address", &from_address)])
        }
    }

    pub fn notification_transaction_failed_title(&self, action: TransactionAction) -> String {
        let action = match action {
            TransactionAction::Transfer => id::notification_action_transfer,
            TransactionAction::Swap => id::notification_action_swap,
            TransactionAction::TokenApproval => id::notification_action_token_approval,
            TransactionAction::Stake => id::notification_action_stake,
            TransactionAction::Unstake => id::notification_action_unstake,
            TransactionAction::Redelegate => id::notification_action_redelegate,
            TransactionAction::ClaimRewards => id::notification_action_claim_rewards,
            TransactionAction::Withdraw => id::notification_action_withdraw,
            TransactionAction::Freeze => id::notification_action_freeze,
            TransactionAction::Unfreeze => id::notification_action_unfreeze,
            TransactionAction::SmartContract => id::notification_action_smart_contract,
        };
        self.format(id::notification_transaction_failed_title, &[("action", &self.format(action, &[]))])
    }

    pub fn notification_onboarding_buy_asset(&self, name: &str) -> (String, String) {
        (
            self.format(id::notification_onboarding_buy_asset_title, &[("name", &name)]),
            self.format(id::notification_onboarding_buy_asset_description, &[("name", &name)]),
        )
    }

    pub fn notification_reward_redeemed_description(&self, points: i32, value: Option<&str>) -> String {
        match value {
            Some(value) => self.format(id::notification_rewards_redeem_points_for_description, &[("points", &points.abs()), ("value", &value)]),
            None => self.format(id::notification_rewards_redeem_points_description, &[("value", &points.abs())]),
        }
    }

    pub fn notification_stake_rewards(&self, value: &str, chain: &str) -> LanguageNotification {
        LanguageNotification {
            title: self.format(id::notification_stake_rewards_title, &[]),
            description: self.format(id::notification_stake_rewards_description, &[("value", &value), ("chain", &chain)]),
        }
    }
}

macro_rules! messages {
    ($($id:ident($($argument:ident: $kind:ty),*);)*) => {
        impl LanguageLocalizer {
            $(
                pub fn $id(&self $(, $argument: $kind)*) -> String {
                    self.format(id::$id, &[$((stringify!($argument), &$argument)),*])
                }
            )*
        }
    };
}

messages! {
    notification_test();
    support_new_message_title();
    notification_rewards_joined_description();
    notification_rewards_redeem_points_title();
    notification_sent_description(address: &str);
    notification_received_description(address: &str);
    notification_token_approval_title(token: &str);
    notification_freeze_title(value: &str);
    notification_unfreeze_title(value: &str);
    notification_stake_title(value: &str);
    notification_unstake_title(value: &str);
    notification_redelegate_title(value: &str);
    notification_withdraw_title(value: &str);
    notification_claim_rewards_title(value: &str);
    notification_swap_title(from_symbol: &str, to_symbol: &str);
    notification_swap_description(from_value: &str, to_value: &str);
    notification_fiat_purchase_title(value: &str);
    notification_fiat_sale_title(value: &str);
    notification_reward_title(value: i32);
    notification_reward_create_username_description();
    notification_reward_invite_description();
    notification_reward_pending_title();
    errors_generic();
    fiat_error_limit_reached();
    fiat_error_region_unavailable();
    fiat_error_quote_unavailable();
    rewards_error_referral_code_not_exist();
    rewards_error_referral_device_already_used();
    rewards_error_referral_cannot_refer_self();
    rewards_error_referral_rewards_not_enabled();
    rewards_error_referral_limit_reached();
    rewards_error_referral_referrer_limit_reached();
    rewards_error_referral_eligibility_expired(value: i64);
    rewards_error_referral_country_ineligible(value: &str);
    rewards_error_username_limit_reached();
    rewards_error_username_too_short(value: usize);
    rewards_error_username_too_long(value: usize);
    rewards_error_username_invalid_characters();
    rewards_error_username_taken();
    rewards_error_username_wallet_has_username();
    rewards_error_redemption_not_eligible();
    rewards_error_redemption_limit_reached();
    rewards_error_redemption_account_too_new();
    rewards_error_redemption_cooldown();
    rewards_error_redemption_not_enough_points();
    rewards_error_redemption_option_not_available();
    rewards_error_redemption_no_username();
    notification_rewards_enabled_title();
    notification_rewards_enabled_description();
    notification_rewards_disabled_title();
    notification_rewards_disabled_description();
    notification_perpetual_long_title(coin: &str);
    notification_perpetual_short_title(coin: &str);
    notification_perpetual_open_description(price: &str);
    notification_perpetual_close_positive_description(pnl: &str);
    notification_perpetual_close_negative_description(pnl: &str);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specific_language() {
        assert_eq!(&LanguageLocalizer::new_with_language("es").notification_test(), "Prueba");
        assert_eq!(&LanguageLocalizer::new_with_language("pt-BR").notification_test(), "Teste");
    }

    #[test]
    fn test_invalid_language_fallback() {
        let localizer = LanguageLocalizer::new_with_language("unknown");
        assert_eq!(&localizer.notification_test(), "Test");
    }

    #[test]
    fn test_pass_argument() {
        let localizer = LanguageLocalizer::new_with_language("es");
        assert_eq!(&localizer.notification_transfer_title(true, "1 BTC"), "💸 Enviado: \u{2068}1 BTC\u{2069}");
    }

    #[test]
    fn test_reward_redeemed_description() {
        let localizer = LanguageLocalizer::new_with_language("en");

        assert_eq!(&localizer.notification_reward_redeemed_description(650, Some("1 USDT")), "💎 Points redeemed: \u{2068}650\u{2069} for \u{2068}1 USDT\u{2069}");
        assert_eq!(&localizer.notification_reward_redeemed_description(650, None), "💎 Points redeemed: \u{2068}650\u{2069}");
    }

    #[test]
    fn test_fiat_purchase_title() {
        let localizer = LanguageLocalizer::new_with_language("en");

        assert_eq!(&localizer.notification_fiat_purchase_title("0.01 ETH"), "🚀 Bought \u{2068}0.01 ETH\u{2069}");
    }

    #[test]
    fn test_price_alert_target_uses_target_and_current_price() {
        let localizer = LanguageLocalizer::new_with_language("en");
        let message = localizer.price_alert_target("Bitcoin (BTC)", "$81,000.00", "$80,954.00", "-0.27%");

        assert_eq!(message.title, "\u{1f3af} \u{2068}Bitcoin (BTC)\u{2069} reached \u{2068}$81,000.00\u{2069}");
        assert_eq!(message.description, "Now at \u{2068}$80,954.00\u{2069} (\u{2068}-0.27%\u{2069}).");
    }

    #[test]
    fn test_rewards_error_referral_eligibility_expired() {
        let english = LanguageLocalizer::new_with_language("en");
        assert_eq!(
            english.rewards_error_referral_eligibility_expired(30),
            "Referral codes only work on a new device and wallet. Maximum age in days: \u{2068}30\u{2069}."
        );
        assert_eq!(
            LanguageLocalizer::new_with_language("ru").rewards_error_referral_eligibility_expired(21),
            "Реферальные коды работают только на новых устройстве и кошельке. Максимальный срок в днях: \u{2068}21\u{2069}."
        );

        for (language, _) in language::SOURCES.iter().filter(|(language, _)| *language != FALLBACK) {
            let text = LanguageLocalizer::new_with_language(language).rewards_error_referral_eligibility_expired(30);
            assert_ne!(text, english.rewards_error_referral_eligibility_expired(30), "{language} reads its own translation, not the English fallback");
        }
    }

    #[test]
    fn test_notification_transaction_failed_title() {
        let english = LanguageLocalizer::new_with_language("en");
        assert_eq!(english.notification_transaction_failed_title(TransactionAction::Swap), "❌ \u{2068}Swap\u{2069}: Failed");
        assert_eq!(english.notification_transaction_failed_title(TransactionAction::TokenApproval), "❌ \u{2068}Token Approval\u{2069}: Failed");
        assert_eq!(english.notification_token_approval_title("USDC"), "✅ Token Approval \u{2068}USDC\u{2069}");
        assert_eq!(english.notification_transaction_failed_title(TransactionAction::ClaimRewards), "❌ \u{2068}Claim Rewards\u{2069}: Failed");
        assert_eq!(english.notification_claim_rewards_title("1 SOL"), "🎁 Claim Rewards \u{2068}1 SOL\u{2069}");
    }
}
