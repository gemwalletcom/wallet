package com.gemwallet.android.data.coordinators.nft

import com.gemwallet.android.application.nft.cases.GetAssetNft
import com.gemwallet.android.application.nft.cases.GetNftAssetDetails
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.stores.GemstoneNftStore
import com.gemwallet.android.domains.nft.NftAssetDetailsData
import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.NFTAssetData
import com.wallet.core.primitives.NFTAssetId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemCollectibleServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class GetNftAssetDetailsImpl(
    private val getSession: GetSession,
    private val getAssetNft: GetAssetNft,
    private val nftStore: GemstoneNftStore,
    private val collectibleService: GemCollectibleServiceInterface,
) : GetNftAssetDetails {
    override fun invoke(assetId: NFTAssetId): Flow<NftAssetDetailsData?> {
        return getSession().filterNotNull()
            .flatMapLatest { session ->
                getAssetNft.getAssetNft(assetId)
                    .combine(nftStore.observeAssetOwnership(session.wallet.id.id, assetId)) { nftData, isOwned -> nftData to isOwned }
                    .map { (nftData, isOwned) ->
                        val asset = nftData.assets.firstOrNull() ?: return@map null
                        val assetData = NFTAssetData(collection = nftData.collection, asset = asset)
                        NftAssetDetailsData(
                            collection = nftData.collection,
                            asset = asset,
                            details = collectibleService.details(session.wallet.type.toGem(), assetData.toGem(), isOwned),
                        )
                    }
            }
            .flowOn(Dispatchers.IO)
    }
}
