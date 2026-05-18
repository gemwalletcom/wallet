package com.gemwallet.android.ui.models.perpetual

import com.gemwallet.android.domains.percentage.PercentageFormatterStyle
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.perpetual.formatPnlWithPercentage
import com.gemwallet.android.domains.price.toValueDirection
import com.gemwallet.android.model.format
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualConfirmData
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualType

object PerpetualConfirmDetailsUIModelFactory {

    fun create(type: PerpetualType): PerpetualConfirmDetailsUIModel? {
        val action: PerpetualConfirmDetailsUIModel.Action
        val data: PerpetualConfirmData
        val direction: PerpetualDirection
        when (type) {
            is PerpetualType.Open -> {
                action = PerpetualConfirmDetailsUIModel.Action.Open
                data = type.content
                direction = data.direction
            }
            is PerpetualType.Close -> {
                action = PerpetualConfirmDetailsUIModel.Action.Close
                data = type.content
                direction = data.direction
            }
            is PerpetualType.Increase -> {
                action = PerpetualConfirmDetailsUIModel.Action.Increase
                data = type.content
                direction = data.direction
            }
            is PerpetualType.Reduce -> {
                action = PerpetualConfirmDetailsUIModel.Action.Reduce
                data = type.content.data
                direction = type.content.positionDirection
            }
            is PerpetualType.Modify -> return null
        }

        return PerpetualConfirmDetailsUIModel(
            action = action,
            direction = direction,
            leverage = data.leverage.toInt(),
            pnl = data.pnl?.let { value ->
                PerpetualConfirmDetailsUIModel.Pnl(
                    text = formatPnlWithPercentage(value, data.marginAmount, dynamicPlace = true),
                    direction = value.toValueDirection(),
                )
            },
            marginText = Currency.USD.format(data.marginAmount, dynamicPlace = true),
            sizeText = Currency.USD.format(data.fiatValue, dynamicPlace = true),
            autoclose = autocloseFrom(data),
            marketPriceText = Currency.USD.format(data.marketPrice, dynamicPlace = true),
            entryPriceText = data.entryPrice?.let { Currency.USD.format(it, dynamicPlace = true) },
            slippageText = data.slippage.formatAsPercentage(style = PercentageFormatterStyle.PercentSignLess),
        )
    }

    private fun autocloseFrom(data: PerpetualConfirmData): PerpetualConfirmDetailsUIModel.Autoclose? {
        val takeProfit = data.takeProfit?.toDoubleOrNull()?.let { Currency.USD.format(it, dynamicPlace = true) }
        val stopLoss = data.stopLoss?.toDoubleOrNull()?.let { Currency.USD.format(it, dynamicPlace = true) }
        if (takeProfit == null && stopLoss == null) return null
        return PerpetualConfirmDetailsUIModel.Autoclose(takeProfit, stopLoss)
    }
}
