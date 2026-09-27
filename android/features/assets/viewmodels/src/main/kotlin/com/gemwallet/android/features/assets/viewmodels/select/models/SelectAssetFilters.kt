package com.gemwallet.android.features.assets.viewmodels.select.models

import com.gemwallet.android.model.NO_QUERY_LIMIT
import com.gemwallet.android.model.Session
import uniffi.gemstone.GemAssetFilter
import uniffi.gemstone.GemSelectAssetScope

class SelectAssetFilters(val session: Session?, val query: String, val limit: Int = NO_QUERY_LIMIT, val scope: GemSelectAssetScope = GemSelectAssetScope.WALLET, val filters: List<GemAssetFilter> = emptyList()) {
    fun queryFilters(): Set<GemAssetFilter> = filters.toSet()
}
