package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.coordinators.GetBuyQuoteUrl
import com.gemwallet.android.data.repositories.buy.BuyRepository

class GetBuyQuoteUrlImpl(
    private val buyRepository: BuyRepository,
) : GetBuyQuoteUrl {

    override suspend fun invoke(quoteId: String, walletId: String): String? {
        return buyRepository.getQuoteUrl(quoteId, walletId)
    }
}
