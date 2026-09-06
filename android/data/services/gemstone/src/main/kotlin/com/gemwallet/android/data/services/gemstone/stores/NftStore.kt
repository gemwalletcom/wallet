package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.data.services.gemstone.nft.toAssetModel
import com.gemwallet.android.data.services.gemstone.nft.toAssetModels
import com.gemwallet.android.data.services.gemstone.nft.toCollectionModel
import com.gemwallet.android.data.services.gemstone.nft.toCollectionModels
import com.gemwallet.android.data.service.store.database.NftDao
import com.gemwallet.android.data.service.store.database.entities.DbNFTAsset
import com.gemwallet.android.data.service.store.database.entities.DbNFTAssociation
import com.gemwallet.android.data.service.store.database.entities.DbNFTCollection
import com.gemwallet.android.ext.toNftAssetId
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.NFTAssetData
import com.wallet.core.primitives.NFTCollection
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.NFTCollectionId
import com.wallet.core.primitives.NFTData
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.first
import uniffi.gemstone.GemNftStore

class GemstoneNftStore(
    private val nftDao: NftDao,
) : GemNftStore {

    override suspend fun saveNfts(walletId: String, data: List<uniffi.gemstone.NftData>) {
        val nftData = data.map { it.toPrimitives() }
        val assets = nftData.flatMap { it.assets }.map { it.toDb() }
        nftDao.updateNft(
            walletId = walletId,
            collections = nftData.map { it.collection.toDb() },
            assets = assets,
            associations = assets.map { DbNFTAssociation(walletId = walletId, assetId = it.id) },
        )
    }

    override suspend fun getAssetData(assetId: String): uniffi.gemstone.NftAssetData? {
        val id = NFTAssetId(assetId)
        val asset = nftDao.getAsset(id).first() ?: return null
        val collection = nftDao.getCollection(asset.collectionId).first() ?: return null
        return NFTAssetData(collection = collection.toCollectionModel(), asset = asset.toAssetModel()).toGem()
    }

    override suspend fun saveAsset(data: uniffi.gemstone.NftAssetData) {
        val assetData = data.toPrimitives()
        nftDao.add(collection = assetData.collection.toDb(), asset = assetData.asset.toDb())
    }

    fun observeNftData(walletId: String): Flow<List<NFTData>> = combine(
        nftDao.getCollections(walletId),
        nftDao.getAssets(walletId),
    ) { collectionEntities, assetEntities ->
        val assets = assetEntities.toAssetModels().groupBy { it.collectionId }
        collectionEntities.toCollectionModels().map { collection -> NFTData(collection, assets[collection.id] ?: emptyList()) }
    }

    fun observeAsset(assetId: NFTAssetId): Flow<DbNFTAsset?> = nftDao.getAsset(assetId)

    suspend fun getWalletAssetIds(walletId: String): List<NFTAssetId> = nftDao.getWalletAssetIds(walletId)

    fun observeCollection(collectionId: NFTCollectionId): Flow<DbNFTCollection?> = nftDao.getCollection(collectionId)
}

private fun NFTCollection.toDb() = DbNFTCollection(
    id = id,
    name = name,
    description = description,
    chain = chain,
    contractAddress = contractAddress,
    imageUrl = images.preview.url,
    previewImageUrl = images.preview.url,
    originalSourceUrl = images.preview.url,
    status = status,
    links = links,
)

private fun NFTAsset.toDb() = DbNFTAsset(
    id = id,
    collectionId = collectionId,
    name = name,
    tokenId = tokenId,
    tokenType = tokenType,
    contractAddress = contractAddress,
    chain = chain,
    description = description,
    imageUrl = images.preview.url,
    previewImageUrl = images.preview.url,
    originalSourceUrl = images.preview.url,
    attributes = attributes,
)
