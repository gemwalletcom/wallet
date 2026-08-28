package com.gemwallet.android.data.repositories.nft

import android.net.http.HttpException
import com.gemwallet.android.cases.nft.GetAssetNft
import com.gemwallet.android.cases.nft.GetListNftCase
import com.gemwallet.android.cases.nft.RefreshNftAsset
import com.gemwallet.android.cases.nft.SyncNfts
import com.gemwallet.android.data.service.store.database.NftDao
import com.gemwallet.android.data.service.store.database.entities.DbNFTAsset
import com.gemwallet.android.data.service.store.database.entities.DbNFTCollection
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import uniffi.gemstone.nftCollectionStatus
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.NFTCollection
import com.wallet.core.primitives.NFTAssetData
import com.wallet.core.primitives.NFTAssetId
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
import uniffi.gemstone.GemNftService

class NftRepository(
    private val nftService: GemNftService,
    private val nftDao: NftDao,
) : SyncNfts, GetListNftCase, GetAssetNft, RefreshNftAsset {

    @Throws(HttpException::class, IOException::class)
    override suspend fun sync(walletId: WalletId) {
        nftService.sync(walletId.id)
    }

    @Throws(HttpException::class, IOException::class)
    override suspend fun refreshNftAsset(wallet: Wallet, assetId: NFTAssetId) {
        nftService.refreshAsset(wallet.id.id, assetId.toIdentifier())
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
            val assetData = nftService.ensureAsset(assetId.toIdentifier()).decodeJson<NFTAssetData>()
            emit(NFTData(collection = assetData.collection, assets = listOf(assetData.asset)))
        }
    }
}

private fun DbNFTAsset.toNftData(collection: DbNFTCollection) = NFTData(
    collection = collection.toCollectionModel(),
    assets = listOf(toAssetModel()),
)

private fun List<DbNFTCollection>.toCollectionModels() = map { it.toCollectionModel() }

internal fun DbNFTCollection.toCollectionModel() = NFTCollection(
    id = id,
    name = name,
    description = description,
    chain = chain,
    contractAddress = contractAddress,
    images = NFTImages(NFTResource(imageUrl, "")),
    status = nftCollectionStatus(status?.toJson()).decodeJson(),
    links = links ?: emptyList(),
)

private fun List<DbNFTAsset>.toAssetModels(): List<NFTAsset> = map { it.toAssetModel() }

internal fun DbNFTAsset.toAssetModel() = NFTAsset(
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
