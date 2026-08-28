package com.gemwallet.android.application.fiat.cases

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatQuote
import com.wallet.core.primitives.FiatQuoteType
import com.wallet.core.primitives.WalletId

interface GetBuyQuotes {
    suspend operator fun invoke(
        walletId: WalletId,
        asset: Asset,
        type: FiatQuoteType,
        currency: Currency,
        amount: Double,
    ): List<FiatQuote>
}
