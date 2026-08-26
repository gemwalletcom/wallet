package com.gemwallet.android.data.repositories.assets

import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.entities.DbBalance
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.data.service.store.database.entities.toUpdateRecord
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.AssetBasic
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAssetStore

class GemstoneAssetStore(
    private val assetsDao: AssetsDao,
) : GemAssetStore {

    override suspend fun getAssetIds(assetIds: List<String>): List<String> = withContext(Dispatchers.IO) {
        assetsDao.getAssetIds(assetIds)
    }

    override suspend fun addAssets(assets: List<String>) = withContext(Dispatchers.IO) {
        val basics = assets.map { it.decodeJson<AssetBasic>() }
        assetsDao.insert(basics.map { it.toRecord() })
        assetsDao.updateBasicAssets(basics.map { it.toUpdateRecord() })
    }

    override suspend fun addMissingBalances(walletId: String, assetIds: List<String>) = withContext(Dispatchers.IO) {
        for (assetId in assetsDao.getAssetIds(assetIds)) {
            assetsDao.insertBalance(
                DbBalance(
                    assetId = assetId,
                    walletId = walletId,
                    isVisible = false,
                    updatedAt = null,
                )
            )
        }
    }
}
