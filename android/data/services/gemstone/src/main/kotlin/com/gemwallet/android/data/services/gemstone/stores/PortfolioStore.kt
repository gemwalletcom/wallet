package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.entities.toAssetInfoModels
import com.gemwallet.android.model.getTotalAmount
import com.gemwallet.android.ext.toIdentifier
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemPortfolioStore
import uniffi.gemstone.PortfolioAsset

class GemstonePortfolioStore(
    private val assetsDao: AssetsDao,
) : GemPortfolioStore {

    override suspend fun getWalletAssets(walletId: String): List<PortfolioAsset> = withContext(Dispatchers.IO) {
        assetsDao.getPortfolioAssets(walletId).toAssetInfoModels().map { assetInfo ->
            PortfolioAsset(
                assetId = assetInfo.asset.id.toIdentifier(),
                value = assetInfo.balance.balance.getTotalAmount(),
            )
        }
    }
}
