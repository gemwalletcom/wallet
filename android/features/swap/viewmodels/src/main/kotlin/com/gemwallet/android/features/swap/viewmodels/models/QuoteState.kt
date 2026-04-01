package com.gemwallet.android.features.swap.viewmodels.models

import com.gemwallet.android.domains.asset.calculateFiat
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.format
import uniffi.gemstone.SwapperQuote
import java.math.BigDecimal

data class QuoteState(
    val quote: SwapperQuote,
    val pay: AssetInfo,
    val receive: AssetInfo,
)

internal val QuoteState.formattedToAmount: String
    get() = receive.asset.format(Crypto(quote.toValue), 8, showSymbol = false)

internal fun QuoteState.validate(): SwapState {
    val availableBalance = pay.balance.balance.available.toBigInteger()
    val fromValue = quote.fromValue
    return if (availableBalance < fromValue.toBigInteger()) {
        SwapState.Error(SwapError.InsufficientBalance(pay.asset.symbol))
    } else {
        SwapState.Ready
    }
}

internal val QuoteState.receiveEquivalent: BigDecimal
    get() = receive.calculateFiat(quote.toValue)
