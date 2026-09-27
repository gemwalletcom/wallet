package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.services.store.database.NftDao
import com.gemwallet.android.data.services.store.database.entities.DbNFTAssociation
import com.gemwallet.android.data.services.store.database.entities.toNFTAsset
import com.gemwallet.android.data.services.store.database.entities.toNFTCollection
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toNftAssetId
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.NFTAssetData
import com.wallet.core.primitives.NFTAssetId
import kotlinx.coroutines.flow.first
import uniffi.gemstone.GemNftStore

class GemstoneNftStore(private val nftDao: NftDao) : GemNftStore {

    override suspend fun saveNfts(walletId: String, data: List<uniffi.gemstone.NftData>) {
        val nftData = data.map { it.toPrimitives() }
        val assets = nftData.flatMap { it.assets }.map { it.toRecord() }
        nftDao.updateNft(
            walletId = walletId,
            collections = nftData.map { it.collection.toRecord() },
            assets = assets,
            associations = assets.map { DbNFTAssociation(walletId = walletId, assetId = it.id) },
        )
    }

    override suspend fun getAssetData(assetId: String): uniffi.gemstone.NftAssetData? {
        val id = NFTAssetId(assetId)
        val asset = nftDao.getAsset(id).first() ?: return null
        val collection = nftDao.getCollection(asset.collectionId).first() ?: return null
        return NFTAssetData(collection = collection.toNFTCollection(), asset = asset.toNFTAsset()).toGem()
    }

    override suspend fun saveAsset(data: uniffi.gemstone.NftAssetData) {
        val assetData = data.toPrimitives()
        nftDao.add(collection = assetData.collection.toRecord(), asset = assetData.asset.toRecord())
    }
}
