package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.coordinators.SearchSelectAssets
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetTag
import kotlinx.coroutines.flow.Flow

class SearchSelectAssetsImpl(
    private val assetsRepository: AssetsRepository,
) : SearchSelectAssets {
    override fun invoke(query: String, tags: List<AssetTag>): Flow<List<AssetInfo>> =
        assetsRepository.search(query, tags, false)
}
