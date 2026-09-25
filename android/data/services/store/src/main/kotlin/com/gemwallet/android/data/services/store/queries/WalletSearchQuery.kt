package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.application.assets.values.AssetsQueryFilter
import com.gemwallet.android.data.services.store.database.AssetListDao
import com.gemwallet.android.data.services.store.database.AssetsDao
import com.gemwallet.android.data.services.store.database.SearchDao
import com.gemwallet.android.data.services.store.database.entities.toAssetInfoModel
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetList
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
class WalletSearchQuery @Inject constructor(private val assetsDao: AssetsDao, private val searchDao: SearchDao, private val assetListDao: AssetListDao) {

    fun assets(walletId: WalletId, searchKey: String, filters: Set<AssetsQueryFilter>, limit: Int): Flow<List<AssetInfo>> = searchDao.hasAssetPriorities(searchKey)
        .map { it > 0 }
        .distinctUntilChanged()
        .flatMapLatest { hasPriority ->
            when (hasPriority) {
                true -> assetsDao.filteredSearch(walletId.id, searchKey, limit, filters, withPriority = true).toAssetInfoModel()
                false -> flowOf(emptyList())
            }
        }

    fun lists(searchKey: String): Flow<List<AssetList>> = assetListDao.searchWithPriority(searchKey.trim()).map { lists -> lists.map { it.toDTO() } }
}
