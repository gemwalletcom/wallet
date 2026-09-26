package com.gemwallet.android.ui.models.actions

sealed interface SettingsAction {
    data object Wallets : SettingsAction
    data object Security : SettingsAction
    data object Notifications : SettingsAction
    data object Preferences : SettingsAction
    data object Connections : SettingsAction
    data object Support : SettingsAction
    data object Rewards : SettingsAction
    data object AboutUs : SettingsAction
    data object Developer : SettingsAction
}
