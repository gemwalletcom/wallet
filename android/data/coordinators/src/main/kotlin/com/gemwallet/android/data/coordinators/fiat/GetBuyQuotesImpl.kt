package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.coordinators.GetBuyQuotes
import com.gemwallet.android.data.repositories.buy.BuyRepository
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.FiatQuote
import com.wallet.core.primitives.FiatQuoteType

class GetBuyQuotesImpl(
    private val buyRepository: BuyRepository,
) : GetBuyQuotes {

    override suspend fun invoke(
        walletId: String,
        asset: Asset,
        type: FiatQuoteType,
        fiatCurrency: String,
        amount: Double,
    ): List<FiatQuote> {
        return buyRepository.getQuotes(walletId, asset, type, fiatCurrency, amount)
    }
}
