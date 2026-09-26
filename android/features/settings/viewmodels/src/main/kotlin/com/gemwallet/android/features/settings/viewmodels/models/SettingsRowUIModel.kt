package com.gemwallet.android.features.settings.viewmodels.models

import com.gemwallet.android.ui.models.actions.SettingsSceneAction
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemRowTap

fun GemListRow.settingsAction(): SettingsSceneAction? = when (tap()) {
    GemRowTap.Wallets -> SettingsSceneAction.Wallets
    GemRowTap.Security -> SettingsSceneAction.Security
    GemRowTap.Notifications -> SettingsSceneAction.Notifications
    GemRowTap.Preferences -> SettingsSceneAction.Preferences
    GemRowTap.WalletConnect -> SettingsSceneAction.Bridges
    GemRowTap.Support -> SettingsSceneAction.Support
    GemRowTap.Rewards -> SettingsSceneAction.Referral
    GemRowTap.AboutUs -> SettingsSceneAction.AboutUs
    GemRowTap.Developer -> SettingsSceneAction.Develop
    else -> null
}
