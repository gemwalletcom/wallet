package com.gemwallet.android.cases.tokens

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency

interface SearchListTokens {
    suspend fun searchList(listId: String, currency: Currency, chains: List<Chain> = emptyList()): Boolean
}
