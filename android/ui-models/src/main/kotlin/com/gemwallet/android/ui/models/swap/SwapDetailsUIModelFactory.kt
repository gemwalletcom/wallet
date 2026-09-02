package com.gemwallet.android.ui.models.swap

import com.gemwallet.android.domains.asset.getSwapProviderIcon
import com.gemwallet.android.domains.percentage.PercentageFormatterStyle
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.swap.AssetRateFormatter
import com.gemwallet.android.domains.swap.buildAssetRatePair
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.swap.SwapPriceImpact
import java.math.BigInteger
import uniffi.gemstone.SwapperProvider
import uniffi.gemstone.SwapperProviderType

object SwapProviderUIModelFactory {
    fun create(
        provider: SwapperProviderType,
        receiveAsset: AssetPriceValue,
        toValue: String,
    ): SwapProviderUIModel {
        return create(
            providerId = provider.id,
            title = provider.protocol,
            receiveAsset = receiveAsset,
            toValue = toValue,
        )
    }

    fun create(
        providerId: SwapperProvider,
        title: String,
        receiveAsset: AssetPriceValue,
        toValue: String,
    ): SwapProviderUIModel {
        val toAmount = Crypto(toValue)
        val fiatValue = receiveAsset.calculateFiat(toValue)

        return SwapProviderUIModel(
            id = providerId,
            title = title,
            icon = providerId.getSwapProviderIcon(),
            amount = ValueFormatter(style = ValueFormatter.Style.Auto)
                .string(toAmount.atomicValue, receiveAsset.asset),
            fiat = receiveAsset.formatFiat(fiatValue),
        )
    }
}

data class SwapDetailsUIModelInput(
    val payAsset: AssetPriceValue,
    val receiveAsset: AssetPriceValue,
    val fromValue: String,
    val toValue: String,
    val provider: SwapProviderUIModel,
    val providers: List<SwapProviderUIModel> = emptyList(),
    val slippageBps: UInt,
    val selectedSlippage: UInt?,
    val etaInSeconds: UInt?,
    val isProviderSelectable: Boolean,
    val priceImpact: SwapPriceImpact? = null,
    val minReceiveValue: BigInteger = BigInteger.ZERO,
    val etaMinutes: UInt? = null,
)

object SwapDetailsUIModelFactory {
    private val rateFormatter = AssetRateFormatter()

    fun create(input: SwapDetailsUIModelInput): SwapDetailsUIModel? {
        val rate = buildAssetRatePair(
            fromAsset = input.payAsset.asset,
            toAsset = input.receiveAsset.asset,
            fromValue = input.fromValue,
            toValue = input.toValue,
            formatter = rateFormatter,
        ) ?: return null

        val slippagePercent = input.slippageBps.toDouble() / 100.0
        val priceImpact = input.priceImpact?.let {
            SwapPriceImpactUIModel(
                type = it.impactType,
                displayText = it.percentage.formatAsPercentage(),
                warningText = it.percentage.formatAsPercentage(style = PercentageFormatterStyle.PercentSignLess),
                isHigh = it.isHigh,
            )
        }

        val toAmount = Crypto(input.toValue)
        val minReceiveAtomic = input.minReceiveValue

        return SwapDetailsUIModel(
            provider = input.provider,
            providers = input.providers,
            rate = rate,
            priceImpact = priceImpact,
            minimumReceive = ValueFormatter(style = ValueFormatter.Style.Auto)
                .string(minReceiveAtomic, input.receiveAsset.asset),
            slippageText = slippagePercent.formatAsPercentage(style = PercentageFormatterStyle.PercentSignLess),
            slippageBps = input.slippageBps,
            selectedSlippage = input.selectedSlippage,
            estimatedTime = input.etaMinutes?.let { "≈ $it min" },
            isProviderSelectable = input.isProviderSelectable,
        )
    }

}

