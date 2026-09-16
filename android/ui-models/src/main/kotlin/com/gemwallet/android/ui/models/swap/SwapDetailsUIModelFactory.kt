package com.gemwallet.android.ui.models.swap

import uniffi.gemstone.GemPercentageStyle
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.swap.AssetRateFormatter
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.model.ValueFormatter
import uniffi.gemstone.SwapPriceImpact
import java.math.BigInteger
import uniffi.gemstone.GemSwapQuoteSummary
import uniffi.gemstone.SwapProvider
import uniffi.gemstone.SwapperProviderType
import uniffi.gemstone.GemValueStyle

object SwapProviderUIModelFactory {
    fun create(
        provider: SwapperProviderType,
        receiveAsset: AssetPriceValue,
        toValue: BigInteger,
    ): SwapProviderUIModel {
        return create(
            providerId = provider.id,
            title = provider.protocol,
            receiveAsset = receiveAsset,
            toValue = toValue,
        )
    }

    fun create(
        providerId: SwapProvider,
        title: String,
        receiveAsset: AssetPriceValue,
        toValue: BigInteger,
    ): SwapProviderUIModel {
        val fiatValue = receiveAsset.calculateFiat(toValue)

        return SwapProviderUIModel(
            id = providerId,
            title = title,
            icon = providerId,
            amount = ValueFormatter(style = GemValueStyle.AUTO)
                .string(toValue, receiveAsset.asset),
            fiat = receiveAsset.formatFiat(fiatValue),
        )
    }
}

data class SwapDetailsUIModelInput(
    val payAsset: AssetPriceValue,
    val receiveAsset: AssetPriceValue,
    val summary: GemSwapQuoteSummary,
    val provider: SwapProviderUIModel,
    val providers: List<SwapProviderUIModel> = emptyList(),
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

        return SwapDetailsUIModel(
            rows = input.summary.rows(priceImpact != null),
            provider = input.provider,
            providers = input.providers,
            rate = rate,
            priceImpact = priceImpact,
            minimumReceive = ValueFormatter(style = GemValueStyle.AUTO)
                .string(minReceiveAtomic, input.receiveAsset.asset),
            slippageText = slippagePercent.formatAsPercentage(style = GemPercentageStyle.UNSIGNED),
            slippageBps = input.slippageBps,
            selectedSlippage = input.selectedSlippage,
            etaInSeconds = input.summary.quote.etaInSeconds,
            isProviderSelectable = input.isProviderSelectable,
        )
    }

}

