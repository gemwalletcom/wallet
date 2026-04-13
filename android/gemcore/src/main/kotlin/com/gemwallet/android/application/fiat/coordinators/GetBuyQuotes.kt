package com.gemwallet.android.application.fiat.coordinators

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.FiatQuote
import com.wallet.core.primitives.FiatQuoteType
import com.wallet.core.primitives.WalletId

interface GetBuyQuotes {
    suspend operator fun invoke(
        walletId: WalletId,
        asset: Asset,
        type: FiatQuoteType,
        fiatCurrency: String,
        amount: Double,
    ): List<FiatQuote>
}
