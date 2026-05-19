package com.gemwallet.android.features.perpetual.views.models

import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.model.CurrencyFormatter
import com.wallet.core.primitives.ChartCandleStick
import com.wallet.core.primitives.Currency

data class CandlePriceInfo(
    val priceValue: String,
    val changedPercentages: String,
    val state: ValueDirection,
) {
    companion object {
        fun from(point: ChartCandleStick?, data: List<ChartCandleStick>): CandlePriceInfo {
            val close = point?.close ?: 0.0
            val open = point?.open ?: 0.0
            val priceValue = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = Currency.USD).string(close)
            val changedPercentages = data.firstOrNull()?.let { periodStart ->
                point?.let { it.close / (periodStart.open * 0.01) - 100.0 }?.formatAsPercentage()
            }.orEmpty()
            val state = when {
                close - open < 0 -> ValueDirection.Down
                close - open > 0 -> ValueDirection.Up
                else -> ValueDirection.None
            }
            return CandlePriceInfo(priceValue, changedPercentages, state)
        }
    }
}
