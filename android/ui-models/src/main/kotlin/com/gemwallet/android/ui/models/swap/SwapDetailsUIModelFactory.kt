package com.gemwallet.android.ui.models.swap

import com.gemwallet.android.domains.asset.calculateFiat
import com.gemwallet.android.domains.asset.formatFiat
import com.gemwallet.android.domains.asset.getSwapProviderIcon
import com.gemwallet.android.domains.swap.AssetRateFormatter
import com.gemwallet.android.domains.swap.toPrimitives
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.format
import uniffi.gemstone.SwapPriceImpact
import uniffi.gemstone.SwapperProvider
import uniffi.gemstone.SwapperProviderType
import uniffi.gemstone.calculateSwapPriceImpact
import java.math.BigDecimal
import java.math.RoundingMode

object SwapProviderUIModelFactory {
    fun create(
        provider: SwapperProviderType,
        receiveAsset: AssetInfo,
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
        receiveAsset: AssetInfo,
        toValue: String,
    ): SwapProviderUIModel {
        val toAmount = Crypto(toValue)
        val fiatValue = receiveAsset.calculateFiat(toValue)

        return SwapProviderUIModel(
            id = providerId,
            title = title,
            icon = providerId.getSwapProviderIcon(),
            amount = receiveAsset.asset.format(toAmount, 2, dynamicPlace = true),
            fiat = receiveAsset.formatFiat(fiatValue),
        )
    }
}

data class SwapDetailsUIModelInput(
    val payAsset: AssetInfo,
    val receiveAsset: AssetInfo,
    val fromValue: String,
    val toValue: String,
    val provider: SwapProviderUIModel,
    val providers: List<SwapProviderUIModel> = emptyList(),
    val slippageBps: UInt,
    val etaInSeconds: UInt?,
    val isProviderSelectable: Boolean,
)

object SwapDetailsUIModelFactory {
    private val rateFormatter = AssetRateFormatter()

    fun create(input: SwapDetailsUIModelInput): SwapDetailsUIModel? {
        return create(input, ::calculateSwapPriceImpact)
    }

    internal fun create(
        input: SwapDetailsUIModelInput,
        priceImpactCalculator: (Double, Double) -> SwapPriceImpact?,
    ): SwapDetailsUIModel? {
        val rate = estimateSwapRate(
            payAsset = input.payAsset,
            receiveAsset = input.receiveAsset,
            fromValue = input.fromValue,
            toValue = input.toValue,
        ) ?: return null

        val slippagePercent = input.slippageBps.toDouble() / 100.0
        val priceImpact = priceImpactCalculator(
            input.payAsset.calculateFiat(input.fromValue).toDouble(),
            input.receiveAsset.calculateFiat(input.toValue).toDouble(),
        )?.let {
            SwapPriceImpactUIModel(
                type = it.impactType.toPrimitives(),
                displayText = it.percentage.formatSwapPercent(),
                warningText = it.percentage.formatSwapPercent(showSign = false),
                isHigh = it.isHigh,
            )
        }

        val toAmount = Crypto(input.toValue)
        val minReceiveAtomic = toAmount.atomicValue.toBigDecimal().let { amount ->
            amount - (amount * BigDecimal.valueOf(slippagePercent / 100.0))
        }.toBigInteger()

        return SwapDetailsUIModel(
            provider = input.provider,
            providers = input.providers,
            rate = rate,
            priceImpact = priceImpact,
            minimumReceive = input.receiveAsset.asset.format(Crypto(minReceiveAtomic), 2, dynamicPlace = true),
            slippageText = slippagePercent.formatSwapPercent(showSign = false),
            estimatedTime = input.etaInSeconds?.formatSwapEta(),
            isProviderSelectable = input.isProviderSelectable,
        )
    }

    private fun estimateSwapRate(
        payAsset: AssetInfo,
        receiveAsset: AssetInfo,
        fromValue: String,
        toValue: String,
    ): SwapRateUIModel? {
        return try {
            val fromAmount = Crypto(fromValue).value(payAsset.asset.decimals)
            val toAmount = Crypto(toValue).value(receiveAsset.asset.decimals)
            if (fromAmount.compareTo(BigDecimal.ZERO) == 0 || toAmount.compareTo(BigDecimal.ZERO) == 0) {
                return null
            }

            SwapRateUIModel(
                forward = rateFormatter.format(
                    fromAsset = payAsset.asset,
                    toAsset = receiveAsset.asset,
                    fromAmount = fromAmount,
                    toAmount = toAmount,
                    direction = AssetRateFormatter.Direction.Direct,
                ),
                reverse = rateFormatter.format(
                    fromAsset = payAsset.asset,
                    toAsset = receiveAsset.asset,
                    fromAmount = fromAmount,
                    toAmount = toAmount,
                    direction = AssetRateFormatter.Direction.Inverse,
                ),
            )
        } catch (_: Throwable) {
            null
        }
    }
}

private fun UInt.formatSwapEta(): String? {
    if (this <= 60u) {
        return null
    }

    val minutes = (this / 60u).toInt().coerceAtLeast(1)
    return "≈ $minutes min"
}

private fun Double.formatSwapPercent(showSign: Boolean = true): String {
    val roundedValue = BigDecimal.valueOf(this)
        .setScale(2, RoundingMode.HALF_UP)
        .toDouble()
    val absoluteValue = kotlin.math.abs(roundedValue)
    val text = "%.2f".format(absoluteValue)
    val sign = when {
        !showSign -> ""
        roundedValue > 0 -> "+"
        roundedValue < 0 -> "-"
        else -> ""
    }

    return "$sign$text%"
}
