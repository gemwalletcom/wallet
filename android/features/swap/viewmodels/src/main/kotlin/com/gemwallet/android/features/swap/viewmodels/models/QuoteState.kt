package com.gemwallet.android.features.swap.viewmodels.models

import com.gemwallet.android.model.AssetInfo
import uniffi.gemstone.SwapperQuote

data class QuoteState(val quote: SwapperQuote, val pay: AssetInfo, val receive: AssetInfo)
