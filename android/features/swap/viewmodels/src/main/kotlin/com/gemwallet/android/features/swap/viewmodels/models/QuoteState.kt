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

internal val QuoteState.validationError: SwapError?
    get() {
        val availableBalance = pay.balance.balance.available.toBigInteger()
        val fromValue = quote.fromValue
        return if (availableBalance < fromValue.toBigInteger()) {
            SwapError.InsufficientBalance(pay.asset.symbol)
        } else {
            null
        }
    }

internal val QuoteState.receiveEquivalent: BigDecimal
    get() = receive.calculateFiat(quote.toValue)
