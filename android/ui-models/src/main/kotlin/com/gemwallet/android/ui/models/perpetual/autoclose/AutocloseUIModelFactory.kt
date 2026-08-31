package com.gemwallet.android.ui.models.perpetual.autoclose

import com.gemwallet.android.domains.percentage.PercentageFormatterStyle
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDataAggregateImpl
import uniffi.gemstone.GemAutocloseEstimator
import com.gemwallet.android.domains.perpetual.autoclose.AutocloseField
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.domains.price.toValueDirection
import com.gemwallet.android.model.CurrencyFormatter
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualPositionData
import com.wallet.core.primitives.TpslType
import uniffi.gemstone.AutocloseValidation
import kotlin.math.abs
import com.gemwallet.android.serializer.toJson

object AutocloseUIModelFactory {

    private val currencyFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Currency, currency = Currency.USD)

    fun create(
        position: PerpetualPositionData,
        takeProfit: AutocloseField,
        stopLoss: AutocloseField,
        confirmEnabled: Boolean,
        showErrors: Boolean = false,
    ): AutocloseUIModel {
        val estimator = GemAutocloseEstimator(
            entryPrice = position.position.entryPrice,
            positionSize = position.position.size,
            direction = position.position.direction.toJson(),
            leverage = position.position.leverage,
        )
        return AutocloseUIModel(
            position = PerpetualPositionDataAggregateImpl(position),
            marketPriceText = currencyFormatter.string(position.perpetual.price),
            entryPriceText = currencyFormatter.string(position.position.entryPrice),
            takeProfit = createField(takeProfit, estimator, showErrors),
            stopLoss = createField(stopLoss, estimator, showErrors),
            confirmEnabled = confirmEnabled,
        )
    }

    fun createField(
        field: AutocloseField,
        estimator: GemAutocloseEstimator,
        showErrors: Boolean = true,
    ): AutocloseUIModel.Field {
        val priceForEstimation = field.price.takeIf { field.validation == AutocloseValidation.VALID }
        val pnl = priceForEstimation?.let { estimator.pnl(it) }
        val roe = priceForEstimation?.let { estimator.roe(it) }
        val isProfit = pnl?.let { it >= 0.0 } ?: (field.type == TpslType.TakeProfit)
        return AutocloseUIModel.Field(
            type = field.type,
            isProfit = isProfit,
            pnlText = pnlText(pnl, roe, estimator.hasSize()),
            pnlDirection = roe?.toValueDirection() ?: ValueDirection.None,
            percentSuggestions = estimator.percentSuggestions().map { it.toInt() },
            validation = if (showErrors) field.validation else AutocloseValidation.VALID,
        )
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
