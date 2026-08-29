package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.entities.toAssetInfoModels
import com.gemwallet.android.model.getTotalAmount
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.PortfolioAsset
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemPortfolioStore

class GemstonePortfolioStore(
    private val assetsDao: AssetsDao,
) : GemPortfolioStore {

    override suspend fun getWalletAssets(walletId: String): List<String> = withContext(Dispatchers.IO) {
        assetsDao.getPortfolioAssets(walletId).toAssetInfoModels().map { assetInfo ->
            PortfolioAsset(
                assetId = assetInfo.asset.id,
                value = assetInfo.balance.balance.getTotalAmount().toString(),
            ).toJson()
        }
    }
}
