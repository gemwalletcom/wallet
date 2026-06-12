package com.gemwallet.android.data.repositories.assets

import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.tokens.toPriorityQuery
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.AssetsPriorityDao
import com.gemwallet.android.data.service.store.database.entities.toAssetInfoModel
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetTag
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
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
    private val assetsPriorityDao: AssetsPriorityDao,
    private val sessionRepository: SessionRepository,
) {

    fun search(query: String, tags: List<AssetTag>, byAllWallets: Boolean): Flow<List<AssetInfo>> {
        val query = tags.toPriorityQuery(query)
        return sessionRepository.currentWalletId().flatMapLatest { walletId ->
            assetsPriorityDao.hasPriorities(query).map { it > 0 }.flatMapLatest { hasPriority ->
                when {
                    byAllWallets && hasPriority -> assetsDao.searchByAllWalletsWithPriority(walletId, query)
                    byAllWallets -> assetsDao.searchByAllWallets(walletId, query)
                    hasPriority -> assetsDao.searchWithPriority(walletId, query)
                    else -> assetsDao.search(walletId, query)
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
        return assetsPriorityDao.hasPriorities(query).map { it > 0 }.flatMapLatest { hasPriority ->
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
