package com.gemwallet.android.features.settings.settings.viewmodels.models

import com.gemwallet.android.ui.models.actions.SettingsSceneAction
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowTitle

fun GemListRow.settingsAction(): SettingsSceneAction? = when (this) {
    is GemListRow.Link -> when (title) {
        GemListRowTitle.WALLETS -> SettingsSceneAction.Wallets
        GemListRowTitle.SECURITY -> SettingsSceneAction.Security
        GemListRowTitle.NOTIFICATIONS -> SettingsSceneAction.Notifications
        GemListRowTitle.PREFERENCES -> SettingsSceneAction.Preferences
        GemListRowTitle.WALLET_CONNECT -> SettingsSceneAction.Bridges
        GemListRowTitle.SUPPORT -> SettingsSceneAction.Support
        GemListRowTitle.REWARDS -> SettingsSceneAction.Referral
        GemListRowTitle.ABOUT_US -> SettingsSceneAction.AboutUs
        GemListRowTitle.DEVELOPER -> SettingsSceneAction.Develop
        else -> null
    }

    else -> null
}
