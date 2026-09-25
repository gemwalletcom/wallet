package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.NftDao
import com.gemwallet.android.data.services.store.database.entities.toAssetModel
import com.gemwallet.android.data.services.store.database.entities.toCollectionModel
import com.gemwallet.android.domains.nft.NFTAssetDetails
import com.wallet.core.primitives.NFTAssetData
import com.wallet.core.primitives.NFTAssetId
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
class NFTAssetQuery @Inject constructor(private val nftDao: NftDao) {

    operator fun invoke(walletId: String, assetId: NFTAssetId): Flow<NFTAssetDetails?> = nftDao.getAsset(assetId).flatMapLatest { asset ->
        asset?.let {
            nftDao.getCollection(asset.collectionId).flatMapLatest { collection ->
                collection?.let {
                    val assetData = NFTAssetData(collection = collection.toCollectionModel(), asset = asset.toAssetModel())
                    nftDao.isAssetOwned(walletId, assetId).map { isOwned -> NFTAssetDetails(assetData = assetData, isOwned = isOwned) }
                } ?: flowOf(null)
            }
        } ?: flowOf(null)
    }
}
