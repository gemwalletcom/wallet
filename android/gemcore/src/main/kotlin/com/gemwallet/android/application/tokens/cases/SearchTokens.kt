package com.gemwallet.android.application.tokens.cases

import com.wallet.core.primitives.AssetId

interface SearchTokens {
    suspend fun search(assetIds: List<AssetId>): Boolean
}
