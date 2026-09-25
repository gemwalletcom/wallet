package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.services.store.database.AssetsDao
import com.gemwallet.android.data.services.store.database.entities.toAssetInfoModels
import com.gemwallet.android.model.toGem
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAssetBalance
import uniffi.gemstone.GemPortfolioStore

class GemstonePortfolioStore(private val assetsDao: AssetsDao) : GemPortfolioStore {

    override suspend fun getWalletBalances(walletId: String): List<GemAssetBalance> = withContext(Dispatchers.IO) {
        assetsDao.getPortfolioAssets(walletId).toAssetInfoModels().map { it.balance.toGem() }
    }
}
