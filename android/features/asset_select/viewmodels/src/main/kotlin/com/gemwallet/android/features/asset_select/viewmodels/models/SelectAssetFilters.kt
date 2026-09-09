package com.gemwallet.android.features.asset_select.viewmodels.models

import com.gemwallet.android.model.NO_QUERY_LIMIT
import com.gemwallet.android.model.Session
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemAssetFilter
import uniffi.gemstone.GemSelectAssetScope

class SelectAssetFilters(
    val session: Session?,
    val query: String,
    val chainFilter: List<Chain>,
    val hasBalance: Boolean,
    val limit: Int = NO_QUERY_LIMIT,
    val scope: GemSelectAssetScope = GemSelectAssetScope.WALLET,
    val filters: List<GemAssetFilter> = emptyList(),
)
