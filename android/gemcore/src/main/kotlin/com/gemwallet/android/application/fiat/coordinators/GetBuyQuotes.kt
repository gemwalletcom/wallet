package com.gemwallet.android.application.fiat.coordinators

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.FiatQuote
import com.wallet.core.primitives.FiatQuoteType

interface GetBuyQuotes {
    suspend operator fun invoke(
        walletId: String,
        asset: Asset,
        type: FiatQuoteType,
        fiatCurrency: String,
        amount: Double,
    ): List<FiatQuote>
}
