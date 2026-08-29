package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.GetSearchLists
import com.gemwallet.android.data.services.gemstone.assets.AssetsSearchService
import com.wallet.core.primitives.AssetList
import kotlinx.coroutines.flow.Flow

class GetSearchListsImpl(
    private val searchService: AssetsSearchService,
) : GetSearchLists {
    override fun getSearchLists(query: String): Flow<List<AssetList>> =
        searchService.searchLists(query)
}
