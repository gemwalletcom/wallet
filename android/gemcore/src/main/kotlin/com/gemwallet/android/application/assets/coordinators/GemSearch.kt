package com.gemwallet.android.application.assets.coordinators

import com.gemwallet.android.domains.search.WalletSearchTag
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.SearchResponse

interface GemSearch {
    suspend fun search(
        query: String,
        chains: List<Chain> = emptyList(),
        scope: WalletSearchTag = WalletSearchTag.All,
    ): SearchResponse
}
