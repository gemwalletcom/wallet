package com.gemwallet.android.features.asset_select.viewmodels.models

import com.gemwallet.android.data.services.gemstone.assets.AssetsSearchService
import com.gemwallet.android.domains.asset.queryFilters
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.NO_QUERY_LIMIT
import uniffi.gemstone.GemAssetAction
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest

@OptIn(ExperimentalCoroutinesApi::class)
open class BaseSelectSearch(
    private val searchService: AssetsSearchService,
    private val action: GemAssetAction? = null,
) : SelectSearch {

    override fun items(filters: Flow<SelectAssetFilters?>): Flow<List<AssetInfo>> {
        return filters.flatMapLatest { filters ->
            searchService.search(
                query = filters?.query.orEmpty(),
                byAllWallets = false,
                limit = filters?.limit ?: NO_QUERY_LIMIT,
                filters = action?.queryFilters().orEmpty(),
            )
        }
    }
}
