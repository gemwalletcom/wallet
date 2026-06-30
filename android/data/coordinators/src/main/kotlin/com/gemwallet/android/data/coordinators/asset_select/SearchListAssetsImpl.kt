package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.coordinators.SearchListAssets
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.model.AssetInfo
import kotlinx.coroutines.flow.Flow

class SearchListAssetsImpl(
    private val assetsRepository: AssetsRepository,
) : SearchListAssets {
    override fun invoke(listId: String, limit: Int): Flow<List<AssetInfo>> =
        assetsRepository.searchListAssets(listId, limit)
}
