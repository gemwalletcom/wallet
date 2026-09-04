package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.entities.DbBalance
import com.gemwallet.android.data.service.store.database.entities.toAssetInfoModel
import com.gemwallet.android.data.service.store.database.entities.toAssetLinksModel
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toAssetLinkRecord
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.data.service.store.database.entities.toUpdateRecord
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetFull
import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.AssetMarket
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAssetStore
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId

class GemstoneAssetStore(
    private val assetsDao: AssetsDao,
) : GemAssetStore {

    override suspend fun getAssetIds(assetIds: List<String>): List<String> = withContext(Dispatchers.IO) {
        assetsDao.getAssetIds(assetIds)
    }

    override suspend fun getAssets(assetIds: List<String>): List<uniffi.gemstone.Asset> =
        assetsDao.getAssetsByIds(assetIds).toDTO().map { it.toGem() }

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
        assetsDao.setSwapEnabled(assetIds)
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

    fun observeAssetsInfo(walletId: String): Flow<List<AssetInfo>> = assetsDao.getAssetsInfo(walletId).toAssetInfoModel()

    fun observeAssetsInfo(walletId: String, assetIds: List<String>): Flow<List<AssetInfo>> =
        assetsDao.getAssetsInfoByIds(walletId, assetIds).toAssetInfoModel()

    fun observeAssetsInfoByChain(walletId: String, chain: Chain): Flow<List<AssetInfo>> =
        assetsDao.getAssetsInfoByChain(walletId, chain).toAssetInfoModel()

    fun observeHiddenAssetsInfoByChain(walletId: String, chain: Chain): Flow<List<AssetInfo>> =
        assetsDao.getHiddenAssetsInfoByChain(walletId, chain).toAssetInfoModel()

    fun observeAssetInfo(walletId: String, assetId: AssetId): Flow<AssetInfo?> =
        assetsDao.getAssetInfo(walletId, assetId.toIdentifier(), assetId.chain).map { it?.toDTO() }

    fun observeAsset(assetId: AssetId): Flow<Asset?> = assetsDao.getAsset(assetId.toIdentifier()).map { it?.toDTO() }

    fun observeTokenInfo(walletId: String, assetId: AssetId): Flow<AssetInfo?> =
        assetsDao.getTokenInfo(walletId, assetId.toIdentifier(), assetId.chain).map { it?.toDTO() }

    fun observeAssetLinks(assetId: AssetId): Flow<List<AssetLink>> = assetsDao.getAssetLinks(assetId.toIdentifier()).toAssetLinksModel()

    fun observeAssetMarket(assetId: AssetId): Flow<AssetMarket?> = assetsDao.getAssetMarket(assetId.toIdentifier()).map { it?.toDTO() }
}
