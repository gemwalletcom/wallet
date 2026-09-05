package com.gemwallet.android.data.coordinators.nft

import com.gemwallet.android.ext.toGem
import uniffi.gemstone.GemCollectibleServiceInterface
import com.gemwallet.android.application.nft.cases.GetNftAssetDetails
import com.gemwallet.android.application.nft.cases.GetAssetNft
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.nft.NftAssetDetailsData
import com.gemwallet.android.ext.getAccount
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.NFTAssetId
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.flowOn

@OptIn(ExperimentalCoroutinesApi::class)
class GetNftAssetDetailsImpl(
    private val getSession: GetSession,
    private val getAssetNft: GetAssetNft,
    private val collectibleService: GemCollectibleServiceInterface,
) : GetNftAssetDetails {

    override fun invoke(assetId: NFTAssetId): Flow<NftAssetDetailsData?> {
        return getSession().filterNotNull()
            .flatMapLatest { session ->
                getAssetNft.getAssetNft(assetId)
                    .map { nftData ->
                        val nftAsset = nftData.assets.firstOrNull() ?: return@map null
                        val chain = nftAsset.chain
                        val account = session.wallet.getAccount(chain) ?: return@map null
                        val links = nftAsset.contractAddress?.let { collectibleService.links(chain.string, it, nftAsset.tokenId) }
                        NftAssetDetailsData(
                            collection = nftData.collection,
                            asset = nftAsset,
                            account = account,
                            canSend = collectibleService.canSend(session.wallet.toGem(), chain.string),
                            contractExplorerLink = links?.contract?.let { BlockExplorerLink(it.name, it.link) },
                            tokenIdExplorerLink = links?.token?.let { BlockExplorerLink(it.name, it.link) },
                        )
                    }
            }
            .flowOn(Dispatchers.IO)
    }
}
