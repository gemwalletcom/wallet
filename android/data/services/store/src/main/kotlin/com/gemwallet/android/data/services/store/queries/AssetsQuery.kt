package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.application.assets.values.AssetsQueryFilter
import com.gemwallet.android.application.assets.values.AssetsQueryScope
import com.gemwallet.android.data.services.store.database.AssetsDao
import com.gemwallet.android.data.services.store.database.SearchDao
import com.gemwallet.android.data.services.store.database.entities.toAssetDataModel
import com.wallet.core.primitives.AssetData
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
class AssetsQuery @Inject constructor(private val assetsDao: AssetsDao, private val searchDao: SearchDao) {

    operator fun invoke(walletId: WalletId): Flow<List<AssetData>> = assetsDao.getAssetsInfo(walletId.id).toAssetDataModel()

    operator fun invoke(walletId: WalletId, chain: Chain): Flow<List<AssetData>> = assetsDao.getAssetsInfoByChain(walletId.id, chain).toAssetDataModel()

    fun hidden(walletId: WalletId, chain: Chain): Flow<List<AssetData>> = assetsDao.getHiddenAssetsInfoByChain(walletId.id, chain).toAssetDataModel()

    operator fun invoke(walletId: WalletId, searchBy: String, scope: AssetsQueryScope, filters: Set<AssetsQueryFilter>, limit: Int): Flow<List<AssetData>> {
        val query = searchBy.trim()
        return searchDao.hasAssetPriorities(query)
            .map { it > 0 }
            .distinctUntilChanged()
            .flatMapLatest { hasPriority -> assetsDao.filteredSearch(walletId.id, query, limit, filters, hasPriority, scope) }
            .toAssetDataModel()
    }
}
