package com.gemwallet.android.features.swap.viewmodels.models

import com.gemwallet.android.domains.asset.calculateFiat
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ValueFormatter
import uniffi.gemstone.SwapperQuote
import java.math.BigDecimal
import uniffi.gemstone.GemValueStyle

data class QuoteState(
    val quote: SwapperQuote,
    val pay: AssetInfo,
    val receive: AssetInfo,
)

internal val QuoteState.formattedToAmount: String
    get() = ValueFormatter(style = GemValueStyle.AUTO)
        .string(quote.toValue, receive.asset.decimals)

internal val QuoteState.receiveEquivalent: BigDecimal
    get() = receive.calculateFiat(quote.toValue)
