package com.gemwallet.android.data.repositories.nft

import android.net.http.HttpException
import com.gemwallet.android.cases.nft.GetAssetNft
import com.gemwallet.android.cases.nft.GetListNftCase
import com.gemwallet.android.cases.nft.NftError
import com.gemwallet.android.cases.nft.SyncNfts
import com.gemwallet.android.data.service.store.database.NftDao
import com.gemwallet.android.data.service.store.database.entities.DbNFTAsset
import com.gemwallet.android.data.service.store.database.entities.DbNFTAssociation
import com.gemwallet.android.data.service.store.database.entities.DbNFTCollection
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.NFTCollection
import com.wallet.core.primitives.NFTData
import com.wallet.core.primitives.NFTImages
import com.wallet.core.primitives.NFTResource
import com.wallet.core.primitives.VerificationStatus
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import okio.IOException

class NftRepository(
    private val gemDeviceApiClient: GemDeviceApiClient,
    private val nftDao: NftDao,
) : SyncNfts, GetListNftCase, GetAssetNft {

    @Throws(HttpException::class, IOException::class)
    override suspend fun syncNfts(wallet: Wallet) {
        val nftData = gemDeviceApiClient.getNFTs(walletId = wallet.id).orEmpty()
        val collections = nftData.map {
            DbNFTCollection(
                id = it.collection.id,
                name = it.collection.name,
                description = it.collection.description,
                chain = it.collection.chain,
                contractAddress = it.collection.contractAddress,
                imageUrl = it.collection.images.preview.url,
                previewImageUrl = it.collection.images.preview.url,
                originalSourceUrl = it.collection.images.preview.url,
                status = it.collection.status,
                links = it.collection.links,
            )
        }
        val assets = nftData.flatMap { item ->
            item.assets.map { asset ->
                DbNFTAsset(
                    id = asset.id,
                    collectionId = item.collection.id,
                    name = asset.name,
                    tokenId = asset.tokenId,
                    tokenType = asset.tokenType,
                    contractAddress = asset.contractAddress,
                    chain = asset.chain,
                    description = asset.description,
                    imageUrl = asset.images.preview.url,
                    previewImageUrl = asset.images.preview.url,
                    originalSourceUrl = asset.images.preview.url,
                    attributes = asset.attributes,
                )
            }
        }
        val associations = assets.map {
            DbNFTAssociation(
                walletId = wallet.id,
                assetId = it.id,
            )
        }
        nftDao.updateNft(
            wallet.id,
            collections,
            assets,
            associations,
        )
    }

    override fun getListNft(collectionId: String?): Flow<List<NFTData>> {
        return combine(
            nftDao.getCollection(),
            nftDao.getAssets(),
        ) { collectionEntities, assetEntities ->
            val assets = assetEntities.toAssetModels().groupBy { it.collectionId }
            val collections = collectionEntities.toCollectionModels()
            collections.map { NFTData(it, assets[it.id] ?: emptyList()) }
                .filter { it.collection.id == (collectionId ?: return@filter true) }
        }
    }

    @OptIn(ExperimentalCoroutinesApi::class)
    @Throws(NftError::class)
    override fun getAssetNft(id: String): Flow<NFTData> = nftDao.getAsset(id).flatMapLatest { assetEntity ->
        assetEntity ?: throw NftError.NotFoundAsset

        nftDao.getCollection(assetEntity.collectionId).map { collection ->
            collection ?: throw NftError.NotFoundCollection

            NFTData(
                collection = collection.toCollectionModel(),
                assets = listOf(assetEntity.toAssetModel()),
            )
        }
    }
}

private fun List<DbNFTCollection>.toCollectionModels() = map { it.toCollectionModel() }

private fun DbNFTCollection.toCollectionModel() = NFTCollection(
    id = this.id,
    name = this.name,
    description = this.description,
    chain = this.chain,
    contractAddress = this.contractAddress,
    images = NFTImages(NFTResource(this.imageUrl, "")),
    status = this.status ?: VerificationStatus.Verified,
    links = this.links ?: emptyList(),
)

private fun List<DbNFTAsset>.toAssetModels(): List<NFTAsset> = map { it.toAssetModel() }

private fun DbNFTAsset.toAssetModel() = NFTAsset(
    id = this.id,
    collectionId = this.collectionId,
    tokenId = this.tokenId,
    tokenType = this.tokenType,
    contractAddress = this.contractAddress,
    name = this.name,
    description = this.description,
    chain = this.chain,
    resource = NFTResource("", ""), // TODO: Handle resources
    images = NFTImages(NFTResource(this.imageUrl, "")),
    attributes = this.attributes ?: emptyList(),
)
