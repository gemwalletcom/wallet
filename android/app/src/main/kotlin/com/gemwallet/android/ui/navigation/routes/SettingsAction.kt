package com.gemwallet.android.ui.navigation.routes

import com.wallet.core.primitives.AssetId

sealed interface SettingsAction {
    data object Currencies : SettingsAction
    data object Networks : SettingsAction
    data object PriceAlerts : SettingsAction
    data class AddPriceAlertTarget(val assetId: AssetId) : SettingsAction
    data class PriceAlertTargetComplete(val message: String) : SettingsAction
    data class Chart(val assetId: AssetId) : SettingsAction
    data object InAppNotifications : SettingsAction
    data class OpenNotificationUrl(val url: String) : SettingsAction
    data object Cancel : SettingsAction
}
