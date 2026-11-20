package com.gemwallet.android.features.price_alerts.presents

import com.wallet.core.primitives.AssetId

sealed interface PriceAlertsAction {
    sealed interface Navigation : PriceAlertsAction

    data object Refresh : PriceAlertsAction
    data class Exclude(val id: String) : PriceAlertsAction

    data object Close : Navigation
    data object Add : Navigation
    data class OpenChart(val assetId: AssetId) : Navigation
}
