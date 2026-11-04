package com.gemwallet.android.features.settings.presents

internal sealed interface NotificationsAction {
    data class SetEnabled(val enabled: Boolean) : NotificationsAction
    data object PriceAlerts : NotificationsAction
    data object Cancel : NotificationsAction
}
