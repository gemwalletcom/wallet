package com.gemwallet.android.features.asset_select.viewmodels.models

import com.gemwallet.android.data.services.gemstone.assets.AssetsSearchService
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.NO_QUERY_LIMIT
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest

@OptIn(ExperimentalCoroutinesApi::class)
class ListSelectSearch(
    private val searchService: AssetsSearchService,
    private val searchKey: String,
) : SelectSearch {

    override fun items(filters: Flow<SelectAssetFilters?>): Flow<List<AssetInfo>> {
        return filters.flatMapLatest { filters ->
            searchService.searchAssetsByKey(searchKey, filters?.limit ?: NO_QUERY_LIMIT)
        }
    }
}
