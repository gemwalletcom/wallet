package com.gemwallet.android.features.settings.viewmodels.models

import com.gemwallet.android.ui.models.actions.SettingsAction
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemRowTap

fun GemListRow.settingsAction(): SettingsAction? = when (tap()) {
    GemRowTap.Wallets -> SettingsAction.Wallets
    GemRowTap.Security -> SettingsAction.Security
    GemRowTap.Notifications -> SettingsAction.Notifications
    GemRowTap.Preferences -> SettingsAction.Preferences
    GemRowTap.WalletConnect -> SettingsAction.Connections
    GemRowTap.Support -> SettingsAction.Support
    GemRowTap.Rewards -> SettingsAction.Rewards
    GemRowTap.AboutUs -> SettingsAction.AboutUs
    GemRowTap.Developer -> SettingsAction.Developer
    else -> null
}
