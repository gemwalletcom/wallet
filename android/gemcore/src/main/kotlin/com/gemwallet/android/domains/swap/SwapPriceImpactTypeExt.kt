package com.gemwallet.android.domains.swap

import com.wallet.core.primitives.swap.SwapPriceImpactType
import uniffi.gemstone.SwapPriceImpactType as GemSwapPriceImpactType

fun GemSwapPriceImpactType.toPrimitives(): SwapPriceImpactType = when (this) {
    GemSwapPriceImpactType.POSITIVE -> SwapPriceImpactType.Positive
    GemSwapPriceImpactType.LOW -> SwapPriceImpactType.Low
    GemSwapPriceImpactType.MEDIUM -> SwapPriceImpactType.Medium
    GemSwapPriceImpactType.HIGH -> SwapPriceImpactType.High
}
