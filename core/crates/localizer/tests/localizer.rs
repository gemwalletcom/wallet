use localizer::LanguageLocalizer;

/// Test that the expected languages and fallback language are
/// available.
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
