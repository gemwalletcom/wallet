package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.GetSearchLists
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.wallet.core.primitives.AssetList
import kotlinx.coroutines.flow.Flow

class GetSearchListsImpl(
    private val assetsRepository: AssetsRepository,
) : GetSearchLists {
    override fun getSearchLists(query: String): Flow<List<AssetList>> =
        assetsRepository.searchLists(query)
}
