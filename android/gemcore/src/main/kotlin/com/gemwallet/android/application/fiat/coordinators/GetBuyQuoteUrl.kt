package com.gemwallet.android.application.fiat.coordinators

interface GetBuyQuoteUrl {
    suspend operator fun invoke(quoteId: String, walletId: String): String?
}
