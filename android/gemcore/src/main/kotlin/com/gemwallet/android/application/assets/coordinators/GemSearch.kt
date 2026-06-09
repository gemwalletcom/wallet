package com.gemwallet.android.application.assets.coordinators

import com.wallet.core.primitives.AssetTag
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.SearchResponse

interface GemSearch {
    suspend fun search(
        query: String,
        chains: List<Chain> = emptyList(),
        tags: List<AssetTag> = emptyList(),
    ): SearchResponse
}
