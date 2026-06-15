package com.gemwallet.android.data.repositories.assets

import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.tokens.toPriorityQuery
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.SearchDao
import com.gemwallet.android.data.service.store.database.entities.toAssetInfoModel
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.NO_QUERY_LIMIT
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetTag
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import javax.inject.Inject
import javax.inject.Singleton

@OptIn(ExperimentalCoroutinesApi::class)
@Singleton
class AssetsSearchService @Inject constructor(
    private val assetsDao: AssetsDao,
    private val searchDao: SearchDao,
    private val sessionRepository: SessionRepository,
) {

    fun search(query: String, tags: List<AssetTag>, byAllWallets: Boolean, limit: Int = NO_QUERY_LIMIT): Flow<List<AssetInfo>> {
        val query = tags.toPriorityQuery(query)
        return sessionRepository.currentWalletId().flatMapLatest { walletId ->
            searchDao.hasAssetPriorities(query).map { it > 0 }.distinctUntilChanged().flatMapLatest { hasPriority ->
                when {
                    byAllWallets && hasPriority -> assetsDao.searchByAllWalletsWithPriority(walletId, query, limit)
                    byAllWallets -> assetsDao.searchByAllWallets(walletId, query, limit)
                    hasPriority -> assetsDao.searchWithPriority(walletId, query, limit)
                    else -> assetsDao.search(walletId, query, limit)
                }
            }
        }
        .toAssetInfoModel()
    }

    fun swapSearch(wallet: Wallet, query: String, byChains: List<Chain>, byAssets: List<AssetId>, tags: List<AssetTag>): Flow<List<AssetInfo>> {
        val query = tags.toPriorityQuery(query)
        val walletChains = wallet.accounts.map { it.chain }
        val includeChains = byChains.filter { walletChains.contains(it) }
        val includeAssetIds = byAssets.filter { walletChains.contains(it.chain) }
        return searchDao.hasAssetPriorities(query).map { it > 0 }.distinctUntilChanged().flatMapLatest { hasPriority ->
                if (hasPriority) {
                    assetsDao.swapSearchWithPriority(wallet.id.id, query, includeChains, includeAssetIds.map { it.toIdentifier() })
                } else {
                    assetsDao.swapSearch(wallet.id.id, query, includeChains, includeAssetIds.map { it.toIdentifier() })
                }
            }
            .toAssetInfoModel()
            .map { assets ->
                assets.filter { asset ->
                    asset.metadata?.isEnabled == true
                }
            }
    }
}
