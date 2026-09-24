package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.AssetsRequestFilter
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.ext.requireChain
import com.wallet.core.primitives.RecentActivityType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemSwapPair
import uniffi.gemstone.GemSwapStore

class GemstoneSwapStore(private val assetsDao: AssetsDao, private val transactionsDao: TransactionsDao) : GemSwapStore {

    override suspend fun getSwapPairs(walletId: String): List<GemSwapPair> = withContext(Dispatchers.IO) {
        transactionsDao.getSwapPairs(walletId).map { GemSwapPair(it.fromAssetId, it.toAssetId) }
    }

    override suspend fun getRecentAssetIds(walletId: String, limit: UInt): List<String> = withContext(Dispatchers.IO) {
        assetsDao.getRecentAssets(
            walletId = walletId,
            type = listOf(RecentActivityType.SwapSelect, RecentActivityType.Swap),
            filters = setOf(AssetsRequestFilter.Enabled, AssetsRequestFilter.Swappable),
            limit = limit.toInt(),
        ).firstOrNull().orEmpty().map { it.asset.id }
    }

    override suspend fun getPayAssetIds(walletId: String, limit: UInt): List<String> = withContext(Dispatchers.IO) {
        assetsDao.search(
            walletId = walletId,
            query = "",
            limit = limit.toInt(),
            enabled = true,
            swappable = true,
        ).firstOrNull().orEmpty().map { it.id }
    }

    override suspend fun getReceiveAssetIds(walletId: String, chains: List<String>, assetIds: List<String>, limit: UInt): List<String> = withContext(Dispatchers.IO) {
        assetsDao.search(
            walletId = walletId,
            query = "",
            limit = limit.toInt(),
            enabled = true,
            swappable = true,
            byChainsOrAssetIds = true,
            chains = chains.map { it.requireChain() },
            assetIds = assetIds,
        ).firstOrNull().orEmpty().map { it.id }
    }
}
