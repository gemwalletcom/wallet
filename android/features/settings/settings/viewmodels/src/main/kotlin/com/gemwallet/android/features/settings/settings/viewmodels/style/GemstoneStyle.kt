package com.gemwallet.android.features.settings.settings.viewmodels.style

import androidx.annotation.DrawableRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemPreferencesRow
import uniffi.gemstone.GemSettingsRow

@DrawableRes
internal fun GemPreferencesRow.icon(): Int? = when (this) {
    GemPreferencesRow.CURRENCY -> R.drawable.settings_currency
    GemPreferencesRow.LANGUAGE -> R.drawable.settings_language
    GemPreferencesRow.APPEARANCE -> R.drawable.settings_appearance
    GemPreferencesRow.NETWORKS -> R.drawable.settings_networks
    GemPreferencesRow.CONTACTS -> R.drawable.settings_contacts
    GemPreferencesRow.PERPETUALS -> R.drawable.settings_pricealert
    GemPreferencesRow.PERPETUAL_LEVERAGE,
    GemPreferencesRow.PERPETUAL_TAKE_PROFIT,
    GemPreferencesRow.PERPETUAL_STOP_LOSS -> null
}

@DrawableRes
internal fun GemSettingsRow.icon(): Int = when (this) {
    GemSettingsRow.WALLETS -> R.drawable.settings_wallets
    GemSettingsRow.SECURITY -> R.drawable.settings_security
    GemSettingsRow.NOTIFICATIONS -> R.drawable.settings_notifications
    GemSettingsRow.PREFERENCES -> R.drawable.settings_preferences
    GemSettingsRow.WALLET_CONNECT -> R.drawable.settings_wc
    GemSettingsRow.SUPPORT -> R.drawable.settings_support
    GemSettingsRow.REWARDS -> R.drawable.settings_wallets
    GemSettingsRow.ABOUT_US -> R.drawable.settings_about_us
    GemSettingsRow.DEVELOPER -> R.drawable.settings_developer
}
