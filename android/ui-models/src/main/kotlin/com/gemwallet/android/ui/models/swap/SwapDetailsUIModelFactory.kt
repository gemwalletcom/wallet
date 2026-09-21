package com.gemwallet.android.ui.models.swap

import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.swap.AssetRateFormatter
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.model.ValueFormatter
import uniffi.gemstone.GemPercentageStyle
import uniffi.gemstone.GemSwapDetailRow
import uniffi.gemstone.GemSwapProviderRow
import uniffi.gemstone.GemSwapQuoteSummary
import uniffi.gemstone.GemValueStyle
import uniffi.gemstone.SwapPriceImpact
import java.math.BigInteger

data class SwapDetailsUIModelInput(
    val payAsset: AssetPriceValue,
    val receiveAsset: AssetPriceValue,
    val summary: GemSwapQuoteSummary,
    val provider: GemSwapProviderRow,
    val providers: List<GemSwapProviderRow> = emptyList(),
    val slippageBps: UInt,
    val selectedSlippage: UInt?,
    val isProviderSelectable: Boolean,
    val priceImpact: SwapPriceImpact? = null,
)

object SwapDetailsUIModelFactory {
    private val rateFormatter = AssetRateFormatter()

    fun create(input: SwapDetailsUIModelInput): SwapDetailsUIModel? {
        val rate = input.summary.rate?.let(rateFormatter::format) ?: return null

        val slippagePercent = input.summary.slippagePercent()
        val priceImpact = input.priceImpact?.let {
            SwapPriceImpactUIModel(
                type = it.impactType,
                displayText = it.percentage.formatAsPercentage(),
                warningText = it.percentage.formatAsPercentage(style = GemPercentageStyle.UNSIGNED),
                isHigh = it.isHigh,
                showsInSummary = it.showsInSummary,
            )
        }

        val minReceiveAtomic = input.summary.minReceiveValue
        val minimumReceive = ValueFormatter(style = GemValueStyle.AUTO).string(minReceiveAtomic, input.receiveAsset.asset)
        val slippageText = slippagePercent.formatAsPercentage(style = GemPercentageStyle.UNSIGNED)
        val rows = input.summary.rows(priceImpact != null).mapNotNull { row ->
            when (row) {
                GemSwapDetailRow.PROVIDER -> null
                GemSwapDetailRow.RATE -> SwapDetailRowUIModel.Rate(rate)
                GemSwapDetailRow.ESTIMATED_TIME -> input.summary.quote.etaInSeconds?.let(SwapDetailRowUIModel::EstimatedTime)
                GemSwapDetailRow.PRICE_IMPACT -> priceImpact?.let(SwapDetailRowUIModel::PriceImpact)
                GemSwapDetailRow.MINIMUM_RECEIVE -> SwapDetailRowUIModel.MinimumReceive(minimumReceive)
                GemSwapDetailRow.SLIPPAGE -> SwapDetailRowUIModel.Slippage(input.selectedSlippage?.let { slippageText })
            }
        }

        return SwapDetailsUIModel(
            rows = rows,
            provider = input.provider,
            providers = input.providers,
            rate = rate,
            priceImpact = priceImpact,
            minimumReceive = minimumReceive,
            slippageText = slippageText,
            slippageBps = input.slippageBps,
            selectedSlippage = input.selectedSlippage,
            etaInSeconds = input.summary.quote.etaInSeconds,
            isProviderSelectable = input.isProviderSelectable,
        )
    }
}
