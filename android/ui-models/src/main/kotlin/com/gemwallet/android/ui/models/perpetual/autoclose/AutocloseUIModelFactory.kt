package com.gemwallet.android.ui.models.perpetual.autoclose

import com.gemwallet.android.domains.percentage.PercentageFormatterStyle
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDataAggregate
import com.gemwallet.android.domains.perpetual.autoclose.AutocloseEstimator
import com.gemwallet.android.domains.perpetual.autoclose.AutocloseField
import com.gemwallet.android.domains.perpetual.formatPnlWithPercentage
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.domains.price.toValueDirection
import com.gemwallet.android.model.CurrencyFormatter
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualPositionData
import com.wallet.core.primitives.TpslType
import kotlin.math.abs

object AutocloseUIModelFactory {

    private val currencyFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Currency, currency = Currency.USD)
    private val marginFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = Currency.USD)

    fun create(
        position: PerpetualPositionData,
        takeProfit: AutocloseField,
        stopLoss: AutocloseField,
        confirmEnabled: Boolean,
        showErrors: Boolean = false,
    ): AutocloseUIModel {
        val estimator = AutocloseEstimator(
            entryPrice = position.position.entryPrice,
            positionSize = position.position.size,
            direction = position.position.direction,
            leverage = position.position.leverage,
        )
        return AutocloseUIModel(
            position = positionSummary(position),
            marketPriceText = currencyFormatter.string(position.perpetual.price),
            entryPriceText = currencyFormatter.string(position.position.entryPrice),
            takeProfit = createField(takeProfit, estimator, showErrors),
            stopLoss = createField(stopLoss, estimator, showErrors),
            confirmEnabled = confirmEnabled,
        )
    }

    fun createField(
        field: AutocloseField,
        estimator: AutocloseEstimator,
        showErrors: Boolean = true,
    ): AutocloseUIModel.Field {
        val priceForEstimation = field.price.takeIf { field.error == null }
        val pnl = priceForEstimation?.let(estimator::pnl)
        val roe = priceForEstimation?.let(estimator::roe)
        val isProfit = pnl?.let { it >= 0.0 } ?: (field.type == TpslType.TakeProfit)
        return AutocloseUIModel.Field(
            type = field.type,
            isProfit = isProfit,
            pnlText = pnlText(pnl, roe, estimator.hasSize),
            pnlDirection = roe?.toValueDirection() ?: ValueDirection.None,
            percentSuggestions = estimator.percentSuggestions,
            error = if (showErrors) field.error else null,
        )
    }

    private fun positionSummary(data: PerpetualPositionData): PerpetualPositionDataAggregate = object : PerpetualPositionDataAggregate {
        override val positionId: String = data.position.id
        override val perpetualId: PerpetualId = data.perpetual.id
        override val asset: Asset = data.asset
        override val name: String = data.perpetual.name
        override val direction: PerpetualDirection = data.position.direction
        override val leverage: Int = data.position.leverage.toInt()
        override val marginAmount: String = marginFormatter.string(data.position.marginAmount)
        override val pnlWithPercentage: String =
            formatPnlWithPercentage(data.position.pnl, data.position.marginAmount)
        override val pnlState: ValueDirection = data.position.pnl.toValueDirection()
    }

    private fun pnlText(pnl: Double?, roe: Double?, hasSize: Boolean): String {
        if (pnl == null || roe == null) return "-"
        val percentText = roe.formatAsPercentage(style = PercentageFormatterStyle.Percent)
        if (!hasSize) return percentText
        val sign = if (pnl >= 0.0) "+" else "-"
        val amount = currencyFormatter.string(abs(pnl))
        return "$sign$amount ($percentText)"
    }
}
