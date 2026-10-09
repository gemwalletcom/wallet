package com.gemwallet.android.features.price_alerts.presents

internal sealed interface PriceAlertsAction {
    data object Refresh : PriceAlertsAction
    data object Close : PriceAlertsAction
    data object Add : PriceAlertsAction
    data class Exclude(val id: String) : PriceAlertsAction
}
