package com.gemwallet.android.features.swap.viewmodels.models

import com.gemwallet.android.domains.asset.calculateFiat
import com.gemwallet.android.model.AssetInfo
import uniffi.gemstone.SwapperQuote
import java.math.BigDecimal

data class QuoteState(val quote: SwapperQuote, val pay: AssetInfo, val receive: AssetInfo)

internal val QuoteState.receiveEquivalent: BigDecimal
    get() = receive.calculateFiat(quote.toValue)
