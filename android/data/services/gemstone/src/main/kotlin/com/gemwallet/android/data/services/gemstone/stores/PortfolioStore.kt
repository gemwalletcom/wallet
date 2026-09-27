package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.services.store.database.AssetsDao
import com.gemwallet.android.data.services.store.database.entities.toAssetDataModels
import com.gemwallet.android.ext.toGem
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.AssetData
import uniffi.gemstone.GemPortfolioStore

class GemstonePortfolioStore(private val assetsDao: AssetsDao) : GemPortfolioStore {

    override suspend fun getPortfolioAssets(walletId: String): List<AssetData> = withContext(Dispatchers.IO) {
        assetsDao.getPortfolioAssets(walletId).toAssetDataModels().map { it.toGem() }
    }
}
