package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.application.assets.values.AssetsQueryFilter
import com.gemwallet.android.application.assets.values.AssetsQueryScope
import com.gemwallet.android.data.services.store.database.AssetsDao
import com.gemwallet.android.data.services.store.database.SearchDao
import com.gemwallet.android.data.services.store.database.entities.toAssetInfoModel
import com.gemwallet.android.model.AssetInfo
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

    operator fun invoke(walletId: WalletId): Flow<List<AssetInfo>> = assetsDao.getAssetsInfo(walletId.id).toAssetInfoModel()

    operator fun invoke(walletId: WalletId, chain: Chain): Flow<List<AssetInfo>> = assetsDao.getAssetsInfoByChain(walletId.id, chain).toAssetInfoModel()

    fun hidden(walletId: WalletId, chain: Chain): Flow<List<AssetInfo>> = assetsDao.getHiddenAssetsInfoByChain(walletId.id, chain).toAssetInfoModel()

    operator fun invoke(walletId: WalletId, searchBy: String, scope: AssetsQueryScope, filters: Set<AssetsQueryFilter>, limit: Int): Flow<List<AssetInfo>> {
        val query = searchBy.trim()
        return searchDao.hasAssetPriorities(query)
            .map { it > 0 }
            .distinctUntilChanged()
            .flatMapLatest { hasPriority ->
                when (scope) {
                    AssetsQueryScope.Wallet -> assetsDao.filteredSearch(walletId.id, query, limit, filters, hasPriority)

                    AssetsQueryScope.AllAssets -> when (hasPriority) {
                        true -> assetsDao.searchByAllWalletsWithPriority(walletId.id, query, limit)
                        false -> assetsDao.searchByAllWallets(walletId.id, query, limit)
                    }
                }
            }
            .toAssetInfoModel()
    }
}
