package com.gemwallet.android.data.coordinators.nft

import com.gemwallet.android.application.nft.cases.GetAssetNft
import com.gemwallet.android.application.nft.cases.GetListNft
import com.gemwallet.android.data.services.gemstone.stores.GemstoneNftStore
import com.gemwallet.android.data.services.gemstone.nft.toNftData
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
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemNftService

class GetListNftImpl(
    private val nftStore: GemstoneNftStore,
) : GetListNft {

    override fun getListNft(walletId: WalletId, collectionId: String?): Flow<List<NFTData>> =
        nftStore.observeNftData(walletId.id).map { items ->
            items.filter { collectionId == null || it.collection.id.toIdentifier() == collectionId }
        }
}

@OptIn(ExperimentalCoroutinesApi::class)
class GetAssetNftImpl(
    private val nftService: GemNftService,
    private val nftStore: GemstoneNftStore,
) : GetAssetNft {

    override fun getAssetNft(assetId: NFTAssetId): Flow<NFTData> {
        return nftStore.observeAsset(assetId).flatMapLatest { asset ->
            if (asset == null) return@flatMapLatest storedAsset(assetId)

            nftStore.observeCollection(asset.collectionId).flatMapLatest { collection ->
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
