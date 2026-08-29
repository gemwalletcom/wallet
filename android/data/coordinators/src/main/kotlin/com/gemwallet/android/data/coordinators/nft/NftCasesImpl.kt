package com.gemwallet.android.data.coordinators.nft

import com.gemwallet.android.cases.nft.GetAssetNft
import com.gemwallet.android.cases.nft.GetListNftCase
import com.gemwallet.android.data.service.store.database.NftDao
import com.gemwallet.android.data.repositories.nft.toAssetModels
import com.gemwallet.android.data.repositories.nft.toCollectionModels
import com.gemwallet.android.data.repositories.nft.toNftData
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.NFTAssetData
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.NFTData
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf
import uniffi.gemstone.GemNftService

class GetListNftImpl(
    private val nftDao: NftDao,
) : GetListNftCase {

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
}

@OptIn(ExperimentalCoroutinesApi::class)
class GetAssetNftImpl(
    private val nftService: GemNftService,
    private val nftDao: NftDao,
) : GetAssetNft {

    override fun getAssetNft(assetId: NFTAssetId): Flow<NFTData> {
        return nftDao.getAsset(assetId).flatMapLatest { asset ->
            if (asset == null) return@flatMapLatest storedAsset(assetId)

            nftDao.getCollection(asset.collectionId).flatMapLatest { collection ->
                collection
                    ?.let { flowOf(asset.toNftData(it)) }
                    ?: storedAsset(assetId)
            }
        }
    }

    private fun storedAsset(assetId: NFTAssetId): Flow<NFTData> {
        return flow {
            val assetData = nftService.ensureAsset(assetId.toIdentifier()).decodeJson<NFTAssetData>()
            emit(NFTData(collection = assetData.collection, assets = listOf(assetData.asset)))
        }
    }
}
