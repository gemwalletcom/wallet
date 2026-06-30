package com.gemwallet.android.cases.tokens

import com.gemwallet.android.domains.search.WalletSearchTag
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency

interface WalletSearchScopeCase {
    suspend fun search(query: String, currency: Currency, chains: List<Chain>, scope: WalletSearchTag): Boolean
}
