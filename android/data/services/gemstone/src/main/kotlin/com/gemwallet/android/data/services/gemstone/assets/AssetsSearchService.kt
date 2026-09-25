package com.gemwallet.android.data.services.gemstone.assets

import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.service.store.database.AssetListDao
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.SearchDao
import com.gemwallet.android.data.service.store.database.entities.DbAssetInfo
import com.gemwallet.android.data.service.store.database.entities.toAssetInfoModel
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.NO_QUERY_LIMIT
import com.gemwallet.android.model.chains
import com.gemwallet.android.model.chainsOrAssetIds
import com.wallet.core.primitives.AssetList
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemAssetFilter
import javax.inject.Inject
import javax.inject.Singleton

@OptIn(ExperimentalCoroutinesApi::class)
@Singleton
class AssetsSearchService @Inject constructor(private val assetsDao: AssetsDao, private val searchDao: SearchDao, private val assetListDao: AssetListDao, private val getCurrentWalletId: GetCurrentWalletId) {

    fun search(query: String, byAllWallets: Boolean, limit: Int = NO_QUERY_LIMIT, filters: Set<GemAssetFilter> = emptySet()): Flow<List<AssetInfo>> {
        val query = query.trim()
        return getCurrentWalletId().flatMapLatest { wallet ->
            val walletId = wallet.id
            searchDao.hasAssetPriorities(query).map { it > 0 }.distinctUntilChanged().flatMapLatest { hasPriority ->
                when {
                    byAllWallets && hasPriority -> assetsDao.searchByAllWalletsWithPriority(walletId, query, limit)
                    byAllWallets -> assetsDao.searchByAllWallets(walletId, query, limit)
                    else -> assetsDao.filteredSearch(walletId, query, limit, filters, hasPriority)
                }
            }
        }
            .toAssetInfoModel()
    }

    fun searchLists(query: String): Flow<List<AssetList>> {
        val key = query.trim()
        return assetListDao.searchWithPriority(key).map { lists -> lists.map { it.toDTO() } }
    }

    fun searchAssetsByKey(searchKey: String, limit: Int = NO_QUERY_LIMIT, filters: Set<GemAssetFilter> = emptySet()): Flow<List<AssetInfo>> = getCurrentWalletId().flatMapLatest { wallet ->
        val walletId = wallet.id
        searchDao.hasAssetPriorities(searchKey).map { it > 0 }.distinctUntilChanged().flatMapLatest { hasPriority ->
            if (hasPriority) {
                assetsDao.filteredSearch(walletId, searchKey, limit, filters, withPriority = true).toAssetInfoModel()
            } else {
                flowOf(emptyList<AssetInfo>())
            }
        }
    }
}

internal fun AssetsDao.filteredSearch(walletId: String, query: String, limit: Int, filters: Set<GemAssetFilter>, withPriority: Boolean): Flow<List<DbAssetInfo>> {
    val scope = filters.chainsOrAssetIds()
    val selectedChains = filters.chains()
    val search = if (withPriority) ::searchWithPriority else ::search
    return search(
        walletId,
        query,
        limit,
        emptyList(),
        GemAssetFilter.Enabled in filters,
        GemAssetFilter.Buyable in filters,
        GemAssetFilter.Sellable in filters,
        GemAssetFilter.Swappable in filters,
        GemAssetFilter.HasBalance in filters,
        GemAssetFilter.HasAvailableBalance in filters,
        scope != null,
        scope?.chains.orEmpty().map { it.requireChain() },
        scope?.assetIds.orEmpty(),
        selectedChains.isNotEmpty(),
        selectedChains,
    )
}
