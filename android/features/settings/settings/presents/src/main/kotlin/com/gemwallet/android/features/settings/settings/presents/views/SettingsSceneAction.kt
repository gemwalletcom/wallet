package com.gemwallet.android.features.settings.settings.presents.views

sealed interface SettingsSceneAction {
    data object Wallets : SettingsSceneAction
    data object Security : SettingsSceneAction
    data object Notifications : SettingsSceneAction
    data object Preferences : SettingsSceneAction
    data object Bridges : SettingsSceneAction
    data object Support : SettingsSceneAction
    data object Referral : SettingsSceneAction
    data object AboutUs : SettingsSceneAction
    data object Develop : SettingsSceneAction
}
