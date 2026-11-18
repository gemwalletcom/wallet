package com.gemwallet.android.features.settings.presents

sealed interface NotificationsAction {
    sealed interface Navigation : NotificationsAction

    data class SetEnabled(val enabled: Boolean) : NotificationsAction

    data object PriceAlerts : Navigation
    data object Cancel : Navigation
}
