package com.gemwallet.android.features.asset_select.viewmodels.models

import com.gemwallet.android.model.NO_QUERY_LIMIT
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemAssetFilter
import uniffi.gemstone.GemSelectAssetScope

fun mockSelectAssetFilters(
    query: String = "",
    chainFilter: List<Chain> = emptyList(),
    hasBalance: Boolean = false,
    limit: Int = NO_QUERY_LIMIT,
    scope: GemSelectAssetScope = GemSelectAssetScope.WALLET,
    filters: List<GemAssetFilter> = emptyList(),
) = SelectAssetFilters(
    session = null,
    query = query,
    chainFilter = chainFilter,
    hasBalance = hasBalance,
    limit = limit,
    scope = scope,
    filters = filters,
)
