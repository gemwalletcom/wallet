package com.gemwallet.android.features.asset_select.viewmodels.models

import com.gemwallet.android.model.NO_QUERY_LIMIT
import uniffi.gemstone.GemAssetFilter
import uniffi.gemstone.GemSelectAssetScope

fun mockSelectAssetFilters(query: String = "", limit: Int = NO_QUERY_LIMIT, scope: GemSelectAssetScope = GemSelectAssetScope.WALLET, filters: List<GemAssetFilter> = emptyList()) = SelectAssetFilters(
    session = null,
    query = query,
    limit = limit,
    scope = scope,
    filters = filters,
)
