package com.gemwallet.android.domains.swap

import com.wallet.core.primitives.swap.SwapPriceImpact
import com.wallet.core.primitives.swap.SwapPriceImpactType
import uniffi.gemstone.GemSwapPriceImpact
import uniffi.gemstone.GemSwapPriceImpactType

fun GemSwapPriceImpact.toPrimitives(): SwapPriceImpact = SwapPriceImpact(
    percentage = percentage,
    impactType = impactType.toPrimitives(),
    isHigh = isHigh,
)

fun GemSwapPriceImpactType.toPrimitives(): SwapPriceImpactType = when (this) {
    GemSwapPriceImpactType.POSITIVE -> SwapPriceImpactType.Positive
    GemSwapPriceImpactType.LOW -> SwapPriceImpactType.Low
    GemSwapPriceImpactType.MEDIUM -> SwapPriceImpactType.Medium
    GemSwapPriceImpactType.HIGH -> SwapPriceImpactType.High
}
