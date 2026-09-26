package com.gemwallet.android.features.price_alerts.presents

import com.wallet.core.primitives.AssetId

internal sealed interface PriceAlertsAction {
    data object Refresh : PriceAlertsAction
    data object Close : PriceAlertsAction
    data object Add : PriceAlertsAction
    data class TogglePriceAlerts(val enabled: Boolean) : PriceAlertsAction
    data class ToggleAutoAlert(val enabled: Boolean) : PriceAlertsAction
    data class Exclude(val id: String) : PriceAlertsAction
    data class OpenChart(val assetId: AssetId) : PriceAlertsAction
    data class SetPriceAlert(val assetId: AssetId) : PriceAlertsAction
}
