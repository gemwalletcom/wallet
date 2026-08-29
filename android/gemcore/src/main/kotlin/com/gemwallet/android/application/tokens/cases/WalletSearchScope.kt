package com.gemwallet.android.application.tokens.cases

import com.gemwallet.android.domains.search.WalletSearchTag
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency

interface WalletSearchScope {
    suspend fun search(query: String, currency: Currency, chains: List<Chain>, scope: WalletSearchTag): Boolean
}
