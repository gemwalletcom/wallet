package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.coordinators.SearchSelectAssets
import com.gemwallet.android.data.repositories.assets.AssetsSearchService
import com.gemwallet.android.model.AssetInfo
import kotlinx.coroutines.flow.Flow

class SearchSelectAssetsImpl(
    private val searchService: AssetsSearchService,
) : SearchSelectAssets {
    override fun invoke(query: String, limit: Int): Flow<List<AssetInfo>> =
        searchService.search(query, false, limit)
}
