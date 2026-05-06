package com.gemwallet.android.application.swap.coordinators

import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemSwapQuoteData
import uniffi.gemstone.SwapperQuote

interface GetSwapQuoteData {
    suspend operator fun invoke(quote: SwapperQuote, wallet: Wallet): GemSwapQuoteData
}
