package com.gemwallet.android.data.coordinators.nft

import com.gemwallet.android.application.nft.cases.GetNftAssetDetails
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.NFTAssetQuery
import com.gemwallet.android.domains.nft.NFTAssetDetails
import com.gemwallet.android.domains.nft.NftAssetDetailsData
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.NFTAssetId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemCollectibleServiceInterface
import uniffi.gemstone.GemNftServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class GetNftAssetDetailsImpl(private val getSession: GetSession, private val nftAssetQuery: NFTAssetQuery, private val nftService: GemNftServiceInterface, private val collectibleService: GemCollectibleServiceInterface) :
    GetNftAssetDetails {
    override fun invoke(assetId: NFTAssetId, canSaveImage: Boolean): Flow<NftAssetDetailsData> = getSession().filterNotNull()
        .flatMapLatest { session ->
            nftAssetQuery(session.wallet.id.id, assetId)
                .flatMapLatest { stored -> stored?.let { flowOf(it) } ?: ensuredAsset(assetId) }
                .map { nftAsset ->
                    NftAssetDetailsData(
                        collection = nftAsset.assetData.collection,
                        asset = nftAsset.assetData.asset,
                        details = collectibleService.details(session.wallet.type.toGem(), nftAsset.assetData.toGem(), nftAsset.isOwned, canSaveImage),
                    )
                }
        }
        .flowOn(Dispatchers.IO)

    private fun ensuredAsset(assetId: NFTAssetId): Flow<NFTAssetDetails> = flow {
        emit(NFTAssetDetails(assetData = nftService.ensureAsset(assetId.toIdentifier()).toPrimitives(), isOwned = false))
    }
}
