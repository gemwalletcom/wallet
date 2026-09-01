package com.gemwallet.android.features.asset_select.viewmodels.models

import com.gemwallet.android.application.asset_select.cases.SearchSelectAssets
import com.gemwallet.android.domains.asset.queryFilters
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.NO_QUERY_LIMIT
import uniffi.gemstone.GemAssetAction
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest

@OptIn(ExperimentalCoroutinesApi::class)
open class BaseSelectSearch(
    private val searchSelectAssets: SearchSelectAssets,
    private val action: GemAssetAction? = null,
) : SelectSearch {

    override fun items(filters: Flow<SelectAssetFilters?>): Flow<List<AssetInfo>> {
        return filters.flatMapLatest { filters ->
            searchSelectAssets(
                filters?.query.orEmpty(),
                filters?.limit ?: NO_QUERY_LIMIT,
                action?.queryFilters().orEmpty(),
            )
        }
    }
}
