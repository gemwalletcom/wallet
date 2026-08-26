package com.gemwallet.android.data.repositories.assets

import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.entities.DbBalance
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toAssetLinkRecord
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.data.service.store.database.entities.toUpdateRecord
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetFull
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAssetStore

class GemstoneAssetStore(
    private val assetsDao: AssetsDao,
) : GemAssetStore {

    override suspend fun getAssetIds(assetIds: List<String>): List<String> = withContext(Dispatchers.IO) {
        assetsDao.getAssetIds(assetIds)
    }

    override suspend fun getAssets(assetIds: List<String>): List<String> = withContext(Dispatchers.IO) {
        assetsDao.getAssetsByIds(assetIds).toDTO().map { it.toJson() }
    }

    override suspend fun saveAssets(assets: List<String>) = withContext(Dispatchers.IO) {
        val basics = assets.map { it.decodeJson<AssetBasic>() }
        assetsDao.insert(basics.map { it.toRecord() })
        assetsDao.updateBasicAssets(basics.map { it.toUpdateRecord() })
    }

    override suspend fun saveAsset(asset: String) = withContext(Dispatchers.IO) {
        val assetFull = asset.decodeJson<AssetFull>()
        assetsDao.upsertAssetMetadata(
            asset = assetFull.toRecord().copy(updatedAt = System.currentTimeMillis()),
            links = assetFull.links.toAssetLinkRecord(assetFull.asset.id),
            market = null,
        )
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
