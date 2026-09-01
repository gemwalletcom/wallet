package com.gemwallet.android.application.tokens.cases

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency

interface SearchTokens {
    suspend fun search(query: String, currency: Currency, chains: List<Chain> = emptyList()): Boolean

    suspend fun search(assetIds: List<AssetId>, currency: Currency): Boolean
}
