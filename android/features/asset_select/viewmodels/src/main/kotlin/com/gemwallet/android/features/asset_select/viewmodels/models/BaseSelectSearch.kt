package com.gemwallet.android.features.asset_select.viewmodels.models

import com.gemwallet.android.data.services.gemstone.assets.AssetsSearchService
import com.gemwallet.android.domains.asset.toQueryFilters
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.NO_QUERY_LIMIT
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import uniffi.gemstone.GemSelectAssetScope

@OptIn(ExperimentalCoroutinesApi::class)
class BaseSelectSearch(
    private val searchService: AssetsSearchService,
) : SelectSearch {

    override fun items(filters: Flow<SelectAssetFilters?>): Flow<List<AssetInfo>> {
        return filters.flatMapLatest { filters ->
            searchService.search(
                query = filters?.query.orEmpty(),
                byAllWallets = filters?.scope == GemSelectAssetScope.ALL_ASSETS,
                limit = filters?.limit ?: NO_QUERY_LIMIT,
                filters = filters?.filters?.toQueryFilters().orEmpty(),
            )
        }
    }
}
