package com.gemwallet.android.data.services.gemstone.assets

import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.service.store.database.AssetListDao
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.entities.toAssetInfoModel
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.model.AssetFilter
import com.gemwallet.android.model.chainsOrAssetIds
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.NO_QUERY_LIMIT
import com.wallet.core.primitives.AssetList
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import javax.inject.Inject
import javax.inject.Singleton

@OptIn(ExperimentalCoroutinesApi::class)
@Singleton
class AssetsSearchService @Inject constructor(
    private val assetsDao: AssetsDao,
    private val assetListDao: AssetListDao,
    private val getCurrentWalletId: GetCurrentWalletId,
) {

    fun search(
        query: String,
        byAllWallets: Boolean,
        limit: Int = NO_QUERY_LIMIT,
        filters: Set<AssetFilter> = emptySet(),
    ): Flow<List<AssetInfo>> {
        val query = query.trim()
        return getCurrentWalletId().flatMapLatest { wallet ->
            assetsDao.search(
                walletId = wallet.id,
                query = query,
                byAllWallets = byAllWallets,
                limit = limit,
                buyable = AssetFilter.Buyable in filters,
                sellable = AssetFilter.Sellable in filters,
                swappable = AssetFilter.Swappable in filters,
                hasBalance = AssetFilter.HasBalance in filters,
                hasAvailableBalance = AssetFilter.HasAvailableBalance in filters,
                byChainsOrAssetIds = filters.chainsOrAssetIds() != null,
                chains = filters.chainsOrAssetIds()?.chains.orEmpty(),
                assetIds = filters.chainsOrAssetIds()?.ids.orEmpty(),
            )
        }
        .toAssetInfoModel()
    }

    fun searchLists(query: String): Flow<List<AssetList>> {
        val key = query.trim()
        return assetListDao.search(key).map { lists -> lists.map { it.toDTO() } }
    }

    fun searchAssetsByKey(searchKey: String, limit: Int = NO_QUERY_LIMIT): Flow<List<AssetInfo>> {
        return getCurrentWalletId()
            .flatMapLatest { wallet -> assetsDao.searchByKey(wallet.id, searchKey, limit) }
            .toAssetInfoModel()
    }

}
