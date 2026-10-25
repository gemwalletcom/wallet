package com.gemwallet.android.features.swap.viewmodels

import uniffi.gemstone.GemSwapPriceImpactRow
import uniffi.gemstone.SwapperQuote

data class SwapPriceImpactConfirmation(val quote: SwapperQuote, val priceImpact: GemSwapPriceImpactRow)
