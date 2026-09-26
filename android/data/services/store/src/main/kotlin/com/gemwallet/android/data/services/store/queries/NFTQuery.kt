package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.NftDao
import com.gemwallet.android.data.services.store.database.entities.toAssetModels
import com.gemwallet.android.data.services.store.database.entities.toCollectionModels
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.NFTData
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import javax.inject.Inject

class NFTQuery @Inject constructor(private val nftDao: NftDao) {

    operator fun invoke(walletId: String, collectionId: String? = null): Flow<List<NFTData>> = combine(
        nftDao.getCollections(walletId),
        nftDao.getAssets(walletId),
    ) { collectionEntities, assetEntities ->
        val assets = assetEntities.toAssetModels().groupBy { it.collectionId }
        collectionEntities.toCollectionModels()
            .map { collection -> NFTData(collection, assets[collection.id] ?: emptyList()) }
            .filter { collectionId == null || it.collection.id.toIdentifier() == collectionId }
    }
}
