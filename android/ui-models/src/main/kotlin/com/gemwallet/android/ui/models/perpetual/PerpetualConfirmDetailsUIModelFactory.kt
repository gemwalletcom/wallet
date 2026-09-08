package com.gemwallet.android.ui.models.perpetual

import com.gemwallet.android.domains.percentage.PercentageFormatterStyle
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.perpetual.formatPnlWithPercentage
import com.gemwallet.android.domains.price.toValueDirection
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.CurrencyFormatter
import com.wallet.core.primitives.Currency
import uniffi.gemstone.GemPerpetualDetails
import uniffi.gemstone.PerpetualConfirmData

object PerpetualConfirmDetailsUIModelFactory {

    private val currencyFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Currency, currency = Currency.USD)

    fun create(details: GemPerpetualDetails): PerpetualConfirmDetailsUIModel {
        val data = details.data
        return PerpetualConfirmDetailsUIModel(
            action = details.action,
            direction = details.direction.toPrimitives(),
            leverage = data.leverage.toInt(),
            pnl = data.pnl?.let { value ->
                PerpetualConfirmDetailsUIModel.Pnl(
                    text = formatPnlWithPercentage(value, data.marginAmount),
                    direction = value.toValueDirection(),
                )
            },
            marginText = currencyFormatter.string(data.marginAmount),
            sizeText = currencyFormatter.string(data.fiatValue),
            autoclose = autocloseFrom(data),
            marketPriceText = currencyFormatter.string(data.marketPrice),
            entryPriceText = data.entryPrice?.let { currencyFormatter.string(it) },
            slippageText = data.slippage.formatAsPercentage(style = PercentageFormatterStyle.PercentSignLess),
        )
    }

    private fun autocloseFrom(data: PerpetualConfirmData): PerpetualConfirmDetailsUIModel.Autoclose? {
        val takeProfit = data.takeProfit?.toDoubleOrNull()?.let { currencyFormatter.string(it) }
        val stopLoss = data.stopLoss?.toDoubleOrNull()?.let { currencyFormatter.string(it) }
        if (takeProfit == null && stopLoss == null) return null
        return PerpetualConfirmDetailsUIModel.Autoclose(takeProfit, stopLoss)
    }
}
