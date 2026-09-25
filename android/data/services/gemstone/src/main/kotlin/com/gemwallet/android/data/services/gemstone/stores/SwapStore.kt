package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.application.assets.values.toQueryFilter
import com.gemwallet.android.data.services.gemstone.assets.filteredSearch
import com.gemwallet.android.data.services.store.database.AssetsDao
import com.gemwallet.android.data.services.store.database.TransactionsDao
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAssetFilter
import uniffi.gemstone.GemSwapPair
import uniffi.gemstone.GemSwapStore
import uniffi.gemstone.RecentActivityType
import uniffi.gemstone.transactionSwapPair

class GemstoneSwapStore(private val assetsDao: AssetsDao, private val transactionsDao: TransactionsDao) : GemSwapStore {

    override suspend fun getSwapPairs(walletId: String): List<GemSwapPair> = withContext(Dispatchers.IO) {
        transactionsDao.getTransactionsByType(WalletId(walletId), TransactionType.Swap).mapNotNull { transactionSwapPair(it.toDTO().toGem()) }
    }

    override suspend fun getRecentAssetIds(walletId: String, types: List<RecentActivityType>, filters: List<GemAssetFilter>, limit: UInt): List<String> = withContext(Dispatchers.IO) {
        assetsDao.getRecentAssets(
            walletId = walletId,
            type = types.map { it.toPrimitives() },
            filters = filters.map { it.toQueryFilter() }.toSet(),
            limit = limit.toInt(),
        ).firstOrNull().orEmpty().map { it.asset.id }
    }

    override suspend fun getAssetIds(walletId: String, filters: List<GemAssetFilter>, limit: UInt): List<String> = withContext(Dispatchers.IO) {
        assetsDao.filteredSearch(walletId = walletId, query = "", limit = limit.toInt(), filters = filters.toSet(), withPriority = false).firstOrNull().orEmpty().map { it.id }
    }
}
