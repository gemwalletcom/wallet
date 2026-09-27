package com.gemwallet.android.ui.navigation.routes

import com.wallet.core.primitives.AssetId
import uniffi.gemstone.UrlAction

sealed interface SettingsAction {
    data object Currencies : SettingsAction
    data object Contacts : SettingsAction
    data object Networks : SettingsAction
    data object PriceAlerts : SettingsAction
    data class SetPriceAlert(val assetId: AssetId) : SettingsAction
    data class SetPriceAlertComplete(val message: String) : SettingsAction
    data class Chart(val assetId: AssetId) : SettingsAction
    data object InAppNotifications : SettingsAction
    data object DeveloperPayments : SettingsAction
    data class Payment(val payload: String) : SettingsAction
    data class OpenNotification(val action: UrlAction) : SettingsAction
    data object Cancel : SettingsAction
}
