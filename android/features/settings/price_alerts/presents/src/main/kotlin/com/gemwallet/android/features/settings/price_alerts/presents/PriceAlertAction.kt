package com.gemwallet.android.features.settings.price_alerts.presents

import com.wallet.core.primitives.AssetId

internal sealed interface PriceAlertAction {
    data object Refresh : PriceAlertAction
    data object Close : PriceAlertAction
    data object Add : PriceAlertAction
    data class TogglePriceAlerts(val enabled: Boolean) : PriceAlertAction
    data class ToggleAutoAlert(val enabled: Boolean) : PriceAlertAction
    data class Exclude(val id: Int) : PriceAlertAction
    data class OpenChart(val assetId: AssetId) : PriceAlertAction
    data class AddTarget(val assetId: AssetId) : PriceAlertAction
}
