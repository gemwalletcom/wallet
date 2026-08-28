package com.gemwallet.android.application.asset_select.coordinators

import com.gemwallet.android.model.AssetFilter
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.NO_QUERY_LIMIT
import kotlinx.coroutines.flow.Flow

interface SearchSelectAssets {
    operator fun invoke(query: String, limit: Int = NO_QUERY_LIMIT, filters: Set<AssetFilter> = emptySet()): Flow<List<AssetInfo>>
}
