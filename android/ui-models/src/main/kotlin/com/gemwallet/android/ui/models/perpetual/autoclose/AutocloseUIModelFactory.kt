package com.gemwallet.android.ui.models.perpetual.autoclose

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import uniffi.gemstone.GemPercentageStyle
import uniffi.gemstone.PriceChangeCalculator
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDataAggregateImpl
import uniffi.gemstone.GemAutocloseEstimator
import uniffi.gemstone.GemAutocloseField
import uniffi.gemstone.GemAutocloseViewState
import com.gemwallet.android.model.text
import uniffi.gemstone.GemValueTone
import com.gemwallet.android.domains.price.tone
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
        takeProfit: GemAutocloseField,
        stopLoss: GemAutocloseField,
        state: GemAutocloseViewState,
    ): AutocloseUIModel {
        val estimator = GemAutocloseEstimator(
            entryPrice = position.position.entryPrice,
            positionSize = position.position.size,
            direction = position.position.direction.toGem(),
            leverage = position.position.leverage,
        )
        return AutocloseUIModel(
            position = PerpetualPositionDataAggregateImpl(position),
            marketPriceText = state.marketPrice.text(),
            entryPriceText = state.entryPrice?.text(),
            takeProfit = createField(takeProfit, estimator, state.showsErrors),
            stopLoss = createField(stopLoss, estimator, state.showsErrors),
            confirmEnabled = state.confirmEnabled,
        )
    }

    fun createField(
        field: GemAutocloseField,
        estimator: GemAutocloseEstimator,
        showErrors: Boolean = true,
    ): AutocloseUIModel.Field {
        val priceForEstimation = field.price.takeIf { field.validation == AutocloseValidation.VALID }
        val pnl = priceForEstimation?.let { estimator.pnl(it) }
        val roe = priceForEstimation?.let { estimator.roe(it) }
        return AutocloseUIModel.Field(
            type = field.tpslType.toPrimitives(),
            isProfit = estimator.isProfit(priceForEstimation, field.tpslType),
            pnlText = pnlText(pnl, roe, estimator.hasSize()),
            pnlDirection = roe?.tone() ?: GemValueTone.NEUTRAL,
            percentSuggestions = estimator.percentSuggestions().map { it.toInt() },
            validation = if (showErrors) field.validation else AutocloseValidation.VALID,
        )
    }

    private val priceChangeCalculator = PriceChangeCalculator()

    private fun pnlText(pnl: Double?, roe: Double?, hasSize: Boolean): String {
        if (pnl == null || roe == null) return "-"
        val percentText = roe.formatAsPercentage(style = GemPercentageStyle.SIGNED)
        if (!hasSize) return percentText
        val amount = priceChangeCalculator.sign(pnl).format(currencyFormatter.string(abs(pnl)))
        return priceChangeCalculator.pnlText(amount, percentText)
    }
}
