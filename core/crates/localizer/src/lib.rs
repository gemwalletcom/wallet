use std::sync::Arc;

use i18n_embed::{
    DefaultLocalizer, LanguageLoader, Localizer,
    fluent::{FluentLanguageLoader, fluent_language_loader},
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n/"]
pub struct LocalizationsEmbed;

pub static LOCALIZATIONS: LocalizationsEmbed = LocalizationsEmbed;

macro_rules! fl {
    ($loader:expr, $message_id:literal) => {{
        i18n_embed_fl::fl!($loader, $message_id)
    }};
    ($loader:expr, $message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($loader, $message_id, $($args), *)
    }};
}

pub struct LanguageLocalizer {
    loader: Arc<FluentLanguageLoader>,
    localizer: DefaultLocalizer<'static>,
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
        let loader = Arc::new(fluent_language_loader!());

        loader.load_fallback_language(&LOCALIZATIONS).expect("Error while loading fallback language");

        let loader_ref: &'static FluentLanguageLoader = unsafe { &*(Arc::as_ptr(&loader) as *const _) };

        let localizer = DefaultLocalizer::new(loader_ref, &LOCALIZATIONS);

        Self { loader, localizer }
    }

    pub fn new_with_language(language: &str) -> Self {
        let localizer = Self::new();
        localizer.select_language(language).unwrap_or_default();
        localizer
    }

    pub fn select_language(&self, language: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let lang_id = language.parse()?;
        self.localizer.select(&[lang_id])?;
        Ok(true)
    }

    pub fn price_alert_up(&self, symbol: &str, price: &str, price_change: &str) -> LanguageNotification {
        LanguageNotification {
            title: fl!(self.loader.as_ref(), "notification_price_alert_up_title", symbol = symbol),
            description: fl!(self.loader.as_ref(), "notification_price_alert_up_description", price = price, price_change = price_change),
        }
    }

    pub fn price_alert_down(&self, symbol: &str, price: &str, price_change: &str) -> LanguageNotification {
        LanguageNotification {
            title: fl!(self.loader.as_ref(), "notification_price_alert_down_title", symbol = symbol),
            description: fl!(self.loader.as_ref(), "notification_price_alert_down_description", price = price, price_change = price_change),
        }
    }

    pub fn price_alert_target(&self, symbol: &str, target_price: &str, current_price: &str, change: &str) -> LanguageNotification {
        LanguageNotification {
            title: fl!(self.loader.as_ref(), "notification_price_alert_target_title", symbol = symbol, price = target_price),
            description: fl!(self.loader.as_ref(), "notification_price_alert_target_description", price = current_price, change = change),
        }
    }

    pub fn price_alert_all_time_high(&self, symbol: &str, price: &str) -> LanguageNotification {
        LanguageNotification {
            title: fl!(self.loader.as_ref(), "notification_price_alert_all_time_high_title", symbol = symbol),
            description: fl!(self.loader.as_ref(), "notification_price_alert_all_time_high_description", symbol = symbol, price = price),
        }
    }

    pub fn test(&self) -> String {
        fl!(self.loader.as_ref(), "notification_test")
    }

    pub fn notification_transfer_title(&self, is_sent: bool, value: &str) -> String {
        if is_sent {
            fl!(self.loader.as_ref(), "notification_sent_title", value = value)
        } else {
            fl!(self.loader.as_ref(), "notification_received_title", value = value)
        }
    }

    pub fn notification_nft_transfer_title(&self, is_sent: bool, value: &str) -> String {
        if is_sent {
            fl!(self.loader.as_ref(), "notification_nft_sent_title", value = value)
        } else {
            fl!(self.loader.as_ref(), "notification_nft_received_title", value = value)
        }
    }

    pub fn notification_transfer_description(&self, is_sent: bool, to_address: &str, from_address: &str) -> String {
        if is_sent {
            fl!(self.loader.as_ref(), "notification_sent_description", address = to_address)
        } else {
            fl!(self.loader.as_ref(), "notification_received_description", address = from_address)
        }
    }

    pub fn notification_sent_description(&self, address: &str) -> String {
        fl!(self.loader.as_ref(), "notification_sent_description", address = address)
    }

    pub fn notification_received_description(&self, address: &str) -> String {
        fl!(self.loader.as_ref(), "notification_received_description", address = address)
    }

    pub fn notification_token_approval_title(&self, token: &str) -> String {
        fl!(self.loader.as_ref(), "notification_token_approval_title", token = token)
    }

    pub fn notification_freeze_title(&self, value: &str) -> String {
        fl!(self.loader.as_ref(), "notification_freeze_title", value = value)
    }

    pub fn notification_unfreeze_title(&self, value: &str) -> String {
        fl!(self.loader.as_ref(), "notification_unfreeze_title", value = value)
    }

    pub fn notification_stake_title(&self, value: &str) -> String {
        fl!(self.loader.as_ref(), "notification_stake_title", value = value)
    }

    pub fn notification_unstake_title(&self, value: &str) -> String {
        fl!(self.loader.as_ref(), "notification_unstake_title", value = value)
    }

    pub fn notification_redelegate_title(&self, value: &str) -> String {
        fl!(self.loader.as_ref(), "notification_redelegate_title", value = value)
    }

    pub fn notification_withdraw_title(&self, value: &str) -> String {
        fl!(self.loader.as_ref(), "notification_withdraw_title", value = value)
    }

    pub fn notification_claim_rewards_title(&self, value: &str) -> String {
        fl!(self.loader.as_ref(), "notification_claim_rewards_title", value = value)
    }

    pub fn notification_swap_title(&self, from_symbol: &str, to_symbol: &str) -> String {
        fl!(self.loader.as_ref(), "notification_swap_title", from_symbol = from_symbol, to_symbol = to_symbol)
    }

    pub fn notification_transaction_failed_title(&self, action: TransactionAction) -> String {
        let loader = self.loader.as_ref();
        let action = match action {
            TransactionAction::Transfer => fl!(loader, "notification_action_transfer"),
            TransactionAction::Swap => fl!(loader, "notification_action_swap"),
            TransactionAction::TokenApproval => fl!(loader, "notification_action_token_approval"),
            TransactionAction::Stake => fl!(loader, "notification_action_stake"),
            TransactionAction::Unstake => fl!(loader, "notification_action_unstake"),
            TransactionAction::Redelegate => fl!(loader, "notification_action_redelegate"),
            TransactionAction::ClaimRewards => fl!(loader, "notification_action_claim_rewards"),
            TransactionAction::Withdraw => fl!(loader, "notification_action_withdraw"),
            TransactionAction::Freeze => fl!(loader, "notification_action_freeze"),
            TransactionAction::Unfreeze => fl!(loader, "notification_action_unfreeze"),
            TransactionAction::SmartContract => fl!(loader, "notification_action_smart_contract"),
        };
        fl!(loader, "notification_transaction_failed_title", action = action)
    }

    pub fn notification_swap_description(&self, from_value: &str, to_value: &str) -> String {
        fl!(self.loader.as_ref(), "notification_swap_description", from_value = from_value, to_value = to_value)
    }

    pub fn notification_onboarding_buy_asset(&self, name: &str) -> (String, String) {
        (
            fl!(self.loader.as_ref(), "notification_onboarding_buy_asset_title", name = name),
            fl!(self.loader.as_ref(), "notification_onboarding_buy_asset_description", name = name),
        )
    }

    pub fn notification_fiat_purchase_title(&self, value: &str) -> String {
        fl!(self.loader.as_ref(), "notification_fiat_purchase_title", value = value)
    }

    pub fn notification_fiat_sale_title(&self, value: &str) -> String {
        fl!(self.loader.as_ref(), "notification_fiat_sale_title", value = value)
    }

    pub fn notification_support_new_message_title(&self) -> String {
        fl!(self.loader.as_ref(), "support_new_message_title")
    }

    pub fn notification_reward_title(&self, points: i32) -> String {
        fl!(self.loader.as_ref(), "notification_reward_title", value = points)
    }

    pub fn notification_reward_create_username_description(&self) -> String {
        fl!(self.loader.as_ref(), "notification_reward_create_username_description")
    }

    pub fn notification_reward_invite_description(&self) -> String {
        fl!(self.loader.as_ref(), "notification_reward_invite_description")
    }

    pub fn notification_reward_joined_description(&self) -> String {
        fl!(self.loader.as_ref(), "notification_rewards_joined_description")
    }

    pub fn notification_reward_pending_title(&self) -> String {
        fl!(self.loader.as_ref(), "notification_reward_pending_title")
    }

    pub fn notification_reward_redeemed_title(&self) -> String {
        fl!(self.loader.as_ref(), "notification_rewards_redeem_points_title")
    }

    pub fn notification_reward_redeemed_description(&self, points: i32, value: Option<&str>) -> String {
        match value {
            Some(value) => {
                fl!(self.loader.as_ref(), "notification_rewards_redeem_points_for_description", points = points.abs(), value = value)
            }
            None => fl!(self.loader.as_ref(), "notification_rewards_redeem_points_description", value = points.abs()),
        }
    }

    pub fn errors_generic(&self) -> String {
        fl!(self.loader.as_ref(), "errors_generic")
    }

    pub fn fiat_error_limit_reached(&self) -> String {
        fl!(self.loader.as_ref(), "fiat_error_limit_reached")
    }

    pub fn fiat_error_region_unavailable(&self) -> String {
        fl!(self.loader.as_ref(), "fiat_error_region_unavailable")
    }

    pub fn fiat_error_quote_unavailable(&self) -> String {
        fl!(self.loader.as_ref(), "fiat_error_quote_unavailable")
    }

    pub fn rewards_error_referral_code_not_exist(&self) -> String {
        fl!(self.loader.as_ref(), "rewards_error_referral_code_not_exist")
    }

    pub fn rewards_error_referral_device_already_used(&self) -> String {
        fl!(self.loader.as_ref(), "rewards_error_referral_device_already_used")
    }

    pub fn rewards_error_referral_cannot_refer_self(&self) -> String {
        fl!(self.loader.as_ref(), "rewards_error_referral_cannot_refer_self")
    }

    pub fn rewards_error_referral_rewards_not_enabled(&self) -> String {
        fl!(self.loader.as_ref(), "rewards_error_referral_rewards_not_enabled")
    }

    pub fn rewards_error_referral_limit_reached(&self) -> String {
        fl!(self.loader.as_ref(), "rewards_error_referral_limit_reached")
    }

    pub fn rewards_error_referral_referrer_limit_reached(&self) -> String {
        fl!(self.loader.as_ref(), "rewards_error_referral_referrer_limit_reached")
    }

    pub fn rewards_error_referral_eligibility_expired(&self, days: i64) -> String {
        fl!(self.loader.as_ref(), "rewards_error_referral_eligibility_expired", value = days)
    }

    pub fn rewards_error_referral_country_ineligible(&self, country: &str) -> String {
        fl!(self.loader.as_ref(), "rewards_error_referral_country_ineligible", value = country)
    }

    pub fn rewards_error_username_limit_reached(&self) -> String {
        fl!(self.loader.as_ref(), "rewards_error_username_limit_reached")
    }

    pub fn rewards_error_username_too_short(&self, min_length: usize) -> String {
        fl!(self.loader.as_ref(), "rewards_error_username_too_short", value = min_length)
    }

    pub fn rewards_error_username_too_long(&self, max_length: usize) -> String {
        fl!(self.loader.as_ref(), "rewards_error_username_too_long", value = max_length)
    }

    pub fn rewards_error_username_invalid_characters(&self) -> String {
        fl!(self.loader.as_ref(), "rewards_error_username_invalid_characters")
    }

    pub fn rewards_error_username_taken(&self) -> String {
        fl!(self.loader.as_ref(), "rewards_error_username_taken")
    }

    pub fn rewards_error_username_wallet_has_username(&self) -> String {
        fl!(self.loader.as_ref(), "rewards_error_username_wallet_has_username")
    }

    pub fn rewards_error_redemption_not_eligible(&self) -> String {
        fl!(self.loader.as_ref(), "rewards_error_redemption_not_eligible")
    }

    pub fn rewards_error_redemption_limit_reached(&self) -> String {
        fl!(self.loader.as_ref(), "rewards_error_redemption_limit_reached")
    }

    pub fn rewards_error_redemption_account_too_new(&self) -> String {
        fl!(self.loader.as_ref(), "rewards_error_redemption_account_too_new")
    }

    pub fn rewards_error_redemption_cooldown(&self) -> String {
        fl!(self.loader.as_ref(), "rewards_error_redemption_cooldown")
    }

    pub fn rewards_error_redemption_not_enough_points(&self) -> String {
        fl!(self.loader.as_ref(), "rewards_error_redemption_not_enough_points")
    }

    pub fn rewards_error_redemption_option_not_available(&self) -> String {
        fl!(self.loader.as_ref(), "rewards_error_redemption_option_not_available")
    }

    pub fn rewards_error_redemption_no_username(&self) -> String {
        fl!(self.loader.as_ref(), "rewards_error_redemption_no_username")
    }

    pub fn notification_rewards_enabled_title(&self) -> String {
        fl!(self.loader.as_ref(), "notification_rewards_enabled_title")
    }

    pub fn notification_rewards_enabled_description(&self) -> String {
        fl!(self.loader.as_ref(), "notification_rewards_enabled_description")
    }

    pub fn notification_rewards_disabled_title(&self) -> String {
        fl!(self.loader.as_ref(), "notification_rewards_disabled_title")
    }

    pub fn notification_rewards_disabled_description(&self) -> String {
        fl!(self.loader.as_ref(), "notification_rewards_disabled_description")
    }

    pub fn notification_perpetual_long_title(&self, coin: &str) -> String {
        fl!(self.loader.as_ref(), "notification_perpetual_long_title", coin = coin)
    }

    pub fn notification_perpetual_short_title(&self, coin: &str) -> String {
        fl!(self.loader.as_ref(), "notification_perpetual_short_title", coin = coin)
    }

    pub fn notification_perpetual_open_description(&self, price: &str) -> String {
        fl!(self.loader.as_ref(), "notification_perpetual_open_description", price = price)
    }

    pub fn notification_perpetual_close_positive_description(&self, pnl: &str) -> String {
        fl!(self.loader.as_ref(), "notification_perpetual_close_positive_description", pnl = pnl)
    }

    pub fn notification_perpetual_close_negative_description(&self, pnl: &str) -> String {
        fl!(self.loader.as_ref(), "notification_perpetual_close_negative_description", pnl = pnl)
    }

    pub fn notification_stake_rewards(&self, value: &str, chain: &str) -> LanguageNotification {
        LanguageNotification {
            title: fl!(self.loader.as_ref(), "notification_stake_rewards_title"),
            description: fl!(self.loader.as_ref(), "notification_stake_rewards_description", value = value, chain = chain),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specific_language() {
        let localizer = LanguageLocalizer::new_with_language("es");
        assert_eq!(&localizer.test(), "Prueba");

        localizer.select_language("pt-BR").unwrap();

        assert_eq!(&localizer.test(), "Teste");
    }

    #[test]
    fn test_invalid_language_fallback() {
        let localizer = LanguageLocalizer::new_with_language("unknown");
        assert_eq!(&localizer.test(), "Test");
    }

    #[test]
    fn test_pass_argument() {
        let localizer = LanguageLocalizer::new_with_language("es");
        assert_eq!(&localizer.notification_transfer_title(true, "1 BTC"), "💸 Enviado: \u{2068}1 BTC\u{2069}");
    }

    #[test]
    fn test_reward_redeemed_description() {
        let localizer = LanguageLocalizer::new_with_language("en");

        assert_eq!(&localizer.notification_reward_redeemed_description(650, Some("1 USDT")), "You redeemed \u{2068}650\u{2069} points for \u{2068}1 USDT\u{2069}.");
        assert_eq!(&localizer.notification_reward_redeemed_description(650, None), "You redeemed \u{2068}650\u{2069} points.");
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
    fn test_the_referral_window_names_the_device_and_wallet_age_in_the_right_plural() {
        let english = LanguageLocalizer::new_with_language("en");
        assert_eq!(
            english.rewards_error_referral_eligibility_expired(30),
            "Referral codes can only be used on a device and wallet set up in the last \u{2068}\u{2068}30\u{2069} days\u{2069}."
        );
        assert_eq!(
            english.rewards_error_referral_eligibility_expired(1),
            "Referral codes can only be used on a device and wallet set up in the last \u{2068}day\u{2069}."
        );

        let russian = LanguageLocalizer::new_with_language("ru");
        assert!(russian.rewards_error_referral_eligibility_expired(30).contains("за последние \u{2068}30\u{2069} дней"));
        assert!(russian.rewards_error_referral_eligibility_expired(3).contains("за последние \u{2068}3\u{2069} дня"));
        assert!(russian.rewards_error_referral_eligibility_expired(21).contains("за последний \u{2068}21\u{2069} день"));

        for language in ["ar", "de", "es", "fa", "fr", "he", "hi", "id", "it", "ja", "ko", "pl", "pt-BR", "ru", "th", "tr", "uk", "vi", "zh-Hans", "zh-Hant"] {
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
