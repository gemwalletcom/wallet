package com.gemwallet.android.data.repositories.nft

import android.net.http.HttpException
import com.gemwallet.android.cases.nft.GetAssetNft
import com.gemwallet.android.cases.nft.GetListNftCase
import com.gemwallet.android.cases.nft.RefreshNftAsset
import com.gemwallet.android.cases.nft.SyncNfts
import com.gemwallet.android.data.service.store.database.NftDao
import com.gemwallet.android.data.service.store.database.entities.DbNFTAsset
import com.gemwallet.android.data.service.store.database.entities.DbNFTAssociation
import com.gemwallet.android.data.service.store.database.entities.DbNFTCollection
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.NFTCollection
import com.wallet.core.primitives.NFTData
import com.wallet.core.primitives.NFTImages
import com.wallet.core.primitives.NFTResource
import com.wallet.core.primitives.VerificationStatus
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf
import okio.IOException

class NftRepository(
    private val gemDeviceApiClient: GemDeviceApiClient,
    private val nftDao: NftDao,
) : SyncNfts, GetListNftCase, GetAssetNft, RefreshNftAsset {

    @Throws(HttpException::class, IOException::class)
    override suspend fun sync(walletId: WalletId) {
        val nftData = gemDeviceApiClient.getNFTs(walletId = walletId).orEmpty()
        val collections = nftData.map { it.collection.toDb() }
        val assets = nftData.flatMap { it.assets }.map { it.toDb() }
        val associations = assets.map { DbNFTAssociation(walletId = walletId.id, assetId = it.id) }
        nftDao.updateNft(walletId.id, collections, assets, associations)
    }

    @Throws(HttpException::class, IOException::class)
    override suspend fun refreshNftAsset(wallet: Wallet, assetId: NFTAssetId) {
        gemDeviceApiClient.refreshNftAsset(wallet.id, assetId.toIdentifier())
    }

    override fun getListNft(walletId: WalletId, collectionId: String?): Flow<List<NFTData>> {
        return combine(
            nftDao.getCollections(walletId.id),
            nftDao.getAssets(walletId.id),
        ) { collectionEntities, assetEntities ->
            val assets = assetEntities.toAssetModels().groupBy { it.collectionId }
            val collections = collectionEntities.toCollectionModels()
            collections.map { collection -> NFTData(collection, assets[collection.id] ?: emptyList()) }
                .filter { collectionId == null || it.collection.id.toIdentifier() == collectionId }
        }
    }

    @OptIn(ExperimentalCoroutinesApi::class)
    override fun getAssetNft(assetId: NFTAssetId): Flow<NFTData> {
        return nftDao.getAsset(assetId).flatMapLatest { asset ->
            if (asset == null) return@flatMapLatest fetchAndAddNftAsset(assetId)

            nftDao.getCollection(asset.collectionId).flatMapLatest { collection ->
                collection
                    ?.let { flowOf(asset.toNftData(it)) }
                    ?: fetchAndAddNftAsset(assetId)
            }
        }
    }

    private fun fetchAndAddNftAsset(assetId: NFTAssetId): Flow<NFTData> {
        return flow {
            val assetData = gemDeviceApiClient.getNFT(assetId.toIdentifier())
            nftDao.add(collection = assetData.collection.toDb(), asset = assetData.asset.toDb())
            emit(NFTData(collection = assetData.collection, assets = listOf(assetData.asset)))
        }
    }
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

private fun DbNFTAsset.toNftData(collection: DbNFTCollection) = NFTData(
    collection = collection.toCollectionModel(),
    assets = listOf(toAssetModel()),
)

private fun List<DbNFTCollection>.toCollectionModels() = map { it.toCollectionModel() }

private fun DbNFTCollection.toCollectionModel() = NFTCollection(
    id = id,
    name = name,
    description = description,
    chain = chain,
    contractAddress = contractAddress,
    images = NFTImages(NFTResource(imageUrl, "")),
    status = status ?: VerificationStatus.Verified,
    links = links ?: emptyList(),
)

private fun List<DbNFTAsset>.toAssetModels(): List<NFTAsset> = map { it.toAssetModel() }

private fun DbNFTAsset.toAssetModel() = NFTAsset(
    id = id,
    collectionId = collectionId,
    tokenId = tokenId,
    tokenType = tokenType,
    contractAddress = contractAddress,
    name = name,
    description = description,
    chain = chain,
    resource = NFTResource("", ""),
    images = NFTImages(NFTResource(imageUrl, "")),
    attributes = attributes ?: emptyList(),
)
