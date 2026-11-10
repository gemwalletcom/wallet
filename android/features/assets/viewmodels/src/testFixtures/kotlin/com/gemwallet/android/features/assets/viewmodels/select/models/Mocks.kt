package com.gemwallet.android.features.assets.viewmodels.select.models

import com.gemwallet.android.model.NO_QUERY_LIMIT
import com.gemwallet.android.model.Session
import uniffi.gemstone.GemAssetFilter
import uniffi.gemstone.GemSelectAssetScope

fun mockSelectAssetFilters(session: Session? = null, query: String = "", limit: Int = NO_QUERY_LIMIT, scope: GemSelectAssetScope = GemSelectAssetScope.WALLET, filters: List<GemAssetFilter> = emptyList()) = SelectAssetFilters(
    session = session,
    query = query,
    limit = limit,
    scope = scope,
    filters = filters,
)
