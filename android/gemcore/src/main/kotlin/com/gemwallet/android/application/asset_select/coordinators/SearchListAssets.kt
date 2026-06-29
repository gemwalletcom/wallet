package com.gemwallet.android.application.asset_select.coordinators

import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.NO_QUERY_LIMIT
import kotlinx.coroutines.flow.Flow

interface SearchListAssets {
    operator fun invoke(listId: String, limit: Int = NO_QUERY_LIMIT): Flow<List<AssetInfo>>
}
