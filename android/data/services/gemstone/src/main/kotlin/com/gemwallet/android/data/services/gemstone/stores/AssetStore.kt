package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.application.assets.values.toQueryFilter
import com.gemwallet.android.data.services.store.database.AssetsDao
import com.gemwallet.android.data.services.store.database.entities.DbBalance
import com.gemwallet.android.data.services.store.database.entities.toAssetBasic
import com.gemwallet.android.data.services.store.database.entities.toAssetDataModels
import com.gemwallet.android.data.services.store.database.entities.toAssetLinkRecord
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.data.services.store.database.entities.toUpdateRecord
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetFull
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAssetFilter
import uniffi.gemstone.GemAssetStore

class GemstoneAssetStore(private val assetsDao: AssetsDao) : GemAssetStore {

    override suspend fun getAssetIds(assetIds: List<String>): List<String> = withContext(Dispatchers.IO) {
        assetsDao.getAssetIds(assetIds)
    }

    override suspend fun getAssets(assetIds: List<String>): List<uniffi.gemstone.Asset> = assetsDao.getAssetsByIds(assetIds).toDTO().map { it.toGem() }

    override suspend fun getWalletAssets(walletId: String, filters: List<GemAssetFilter>): List<uniffi.gemstone.Asset> = withContext(Dispatchers.IO) {
        assetsDao.getWalletAssets(walletId, filters.map { it.toQueryFilter() }.toSet()).toAssetDataModels().map { it.asset.toGem() }
    }

    override suspend fun getAssetBasics(assetIds: List<String>): List<uniffi.gemstone.AssetBasic> = withContext(Dispatchers.IO) {
        assetsDao.getAssetsByIds(assetIds).mapNotNull { it.toAssetBasic()?.toGem() }
    }

    override suspend fun saveAssets(assets: List<uniffi.gemstone.AssetBasic>) = withContext(Dispatchers.IO) {
        val basics = assets.map { it.toPrimitives() }
        assetsDao.insert(basics.map { it.toRecord() })
        assetsDao.updateBasicAssets(basics.map { it.toUpdateRecord() })
    }

    override suspend fun saveAsset(asset: uniffi.gemstone.AssetFull) = withContext(Dispatchers.IO) {
        val assetFull = asset.toPrimitives()
        assetsDao.upsertAssetMetadata(
            asset = assetFull.toRecord(),
            links = assetFull.links.toAssetLinkRecord(assetFull.asset.id),
            market = null,
        )
    }

    override suspend fun addBalances(walletId: String, assetIds: List<String>, enabled: Boolean) = withContext(Dispatchers.IO) {
        assetsDao.insertBalances(assetIds.map { balanceRecord(walletId, it, enabled) })
    }

    override suspend fun setBuyableAssets(assetIds: List<String>) = withContext(Dispatchers.IO) {
        assetsDao.setBuyableAssets(assetIds)
    }

    override suspend fun setSellableAssets(assetIds: List<String>) = withContext(Dispatchers.IO) {
        assetsDao.setSellableAssets(assetIds)
    }

    override suspend fun setSwappableAssets(assetIds: List<String>) = withContext(Dispatchers.IO) {
        assetsDao.setSwappableAssets(assetIds)
    }

    override suspend fun setStakeableAssets(assetIds: List<String>) = withContext(Dispatchers.IO) {
        assetsDao.setStakeEnabled(assetIds)
    }

    override suspend fun addMissingBalances(walletId: String, assetIds: List<String>) = withContext(Dispatchers.IO) {
        assetsDao.insertBalances(assetIds.map { balanceRecord(walletId, it, false) })
    }

    private fun balanceRecord(walletId: String, assetId: String, isVisible: Boolean) = DbBalance(
        assetId = AssetId(assetId).toIdentifier(),
        walletId = walletId,
        isVisible = isVisible,
        updatedAt = null,
    )
}
