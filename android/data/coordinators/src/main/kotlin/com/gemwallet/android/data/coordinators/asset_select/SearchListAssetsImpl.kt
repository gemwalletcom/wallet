package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.cases.SearchListAssets
import com.gemwallet.android.data.repositories.assets.AssetsSearchService
import com.gemwallet.android.model.AssetInfo
import kotlinx.coroutines.flow.Flow

class SearchListAssetsImpl(
    private val searchService: AssetsSearchService,
) : SearchListAssets {
    override fun invoke(listId: String, limit: Int): Flow<List<AssetInfo>> =
        searchService.searchListAssets(listId, limit)
}
