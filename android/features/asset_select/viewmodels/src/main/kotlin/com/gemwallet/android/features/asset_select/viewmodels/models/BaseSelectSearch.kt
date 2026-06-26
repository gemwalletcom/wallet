package com.gemwallet.android.features.asset_select.viewmodels.models

import com.gemwallet.android.application.asset_select.coordinators.SearchSelectAssets
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.NO_QUERY_LIMIT
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest

@OptIn(ExperimentalCoroutinesApi::class)
open class BaseSelectSearch(
    private val searchSelectAssets: SearchSelectAssets,
) : SelectSearch {

    override fun items(filters: Flow<SelectAssetFilters?>): Flow<List<AssetInfo>> {
        return filters.flatMapLatest { filters ->
            searchSelectAssets(
                filters?.query.orEmpty(),
                filters?.tag?.let { listOf(it) } ?: emptyList(),
                filters?.limit ?: NO_QUERY_LIMIT,
            )
        }
    }
}
