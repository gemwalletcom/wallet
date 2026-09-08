package com.gemwallet.android.ui.models.perpetual

import com.gemwallet.android.domains.percentage.PercentageFormatterStyle
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.perpetual.formatPnlWithPercentage
import com.gemwallet.android.domains.price.toValueDirection
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.CurrencyFormatter
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualDirection
import uniffi.gemstone.PerpetualConfirmData
import uniffi.gemstone.PerpetualType

object PerpetualConfirmDetailsUIModelFactory {

    private val currencyFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Currency, currency = Currency.USD)

    fun create(type: PerpetualType): PerpetualConfirmDetailsUIModel? {
        val action: PerpetualConfirmDetailsUIModel.Action
        val data: PerpetualConfirmData
        val direction: PerpetualDirection
        when (type) {
            is PerpetualType.Open -> {
                action = PerpetualConfirmDetailsUIModel.Action.Open
                data = type.data
                direction = data.direction.toPrimitives()
            }
            is PerpetualType.Close -> {
                action = PerpetualConfirmDetailsUIModel.Action.Close
                data = type.data
                direction = data.direction.toPrimitives()
            }
            is PerpetualType.Increase -> {
                action = PerpetualConfirmDetailsUIModel.Action.Increase
                data = type.data
                direction = data.direction.toPrimitives()
            }
            is PerpetualType.Reduce -> {
                action = PerpetualConfirmDetailsUIModel.Action.Reduce
                data = type.data.data
                direction = type.data.positionDirection.toPrimitives()
            }
            is PerpetualType.Modify -> return null
        }

        return PerpetualConfirmDetailsUIModel(
            action = action,
            direction = direction,
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
