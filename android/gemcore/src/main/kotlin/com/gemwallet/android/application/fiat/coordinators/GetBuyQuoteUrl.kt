package com.gemwallet.android.application.fiat.coordinators

import com.wallet.core.primitives.WalletId

interface GetBuyQuoteUrl {
    suspend operator fun invoke(quoteId: String, walletId: WalletId): String?
}
