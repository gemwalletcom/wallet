package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.service.store.database.entities.DbAssetInfo
import com.gemwallet.android.model.AssetFilter
import com.wallet.core.primitives.RecentActivityType
import com.gemwallet.android.ext.toChain
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemSwapPair
import uniffi.gemstone.GemSwapStore

class GemstoneSwapStore(
    private val assetsDao: AssetsDao,
    private val transactionsDao: TransactionsDao,
) : GemSwapStore {

    override suspend fun getSwapPairs(walletId: String): List<GemSwapPair> = withContext(Dispatchers.IO) {
        transactionsDao.getSwapPairs(walletId).map { GemSwapPair(it.fromAssetId, it.toAssetId) }
    }

    override suspend fun getRecentAssetIds(walletId: String): List<String> = withContext(Dispatchers.IO) {
        assetsDao.getRecentAssets(
            walletId = walletId,
            type = listOf(RecentActivityType.SwapSelect, RecentActivityType.Swap),
            filters = setOf(AssetFilter.Swappable),
        ).firstOrNull().orEmpty().map { it.asset.id }
    }

    override suspend fun getPayAssetIds(walletId: String): List<String> = withContext(Dispatchers.IO) {
        assetsDao.getAssetsInfo(walletId).firstOrNull().orEmpty()
            .filter { it.isSwapEnabled }
            .sortedWith(compareByDescending<DbAssetInfo> { it.balanceFiatTotalAmount ?: 0.0 }.thenByDescending { it.assetRank })
            .map { it.id }
    }

    override suspend fun getReceiveAssetIds(walletId: String, chains: List<String>, assetIds: List<String>): List<String> = withContext(Dispatchers.IO) {
        assetsDao.swapSearch(walletId, "", chains.mapNotNull { it.toChain() }, assetIds).firstOrNull().orEmpty().map { it.id }
    }
}
