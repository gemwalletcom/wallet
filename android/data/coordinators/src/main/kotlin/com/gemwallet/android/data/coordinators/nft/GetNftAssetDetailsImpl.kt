package com.gemwallet.android.data.coordinators.nft

import com.gemwallet.android.application.nft.coordinators.GetNftAssetDetails
import com.gemwallet.android.cases.nft.GetAssetNft
import com.gemwallet.android.cases.nodes.GetCurrentBlockExplorer
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.domains.nft.NftAssetDetailsData
import com.gemwallet.android.ext.getAccount
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.BlockExplorerLink
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import uniffi.gemstone.Explorer

@OptIn(ExperimentalCoroutinesApi::class)
class GetNftAssetDetailsImpl(
    private val sessionRepository: SessionRepository,
    private val getAssetNft: GetAssetNft,
    private val getCurrentBlockExplorer: GetCurrentBlockExplorer,
) : GetNftAssetDetails {

    override fun invoke(assetId: AssetId): Flow<NftAssetDetailsData?> {
        return sessionRepository.session().filterNotNull()
            .flatMapLatest { session ->
                getAssetNft.getAssetNft(assetId)
                    .filterNotNull()
                    .map { nftData ->
                        val nftAsset = nftData.assets.first()
                        val chain = nftAsset.chain
                        val explorerName = getCurrentBlockExplorer.getCurrentBlockExplorer(chain)
                        val chainExplorer = Explorer(chain.string)
                        NftAssetDetailsData(
                            collection = nftData.collection,
                            asset = nftAsset,
                            account = session.wallet.getAccount(chain)!!,
                            contractExplorerLink = nftAsset.contractAddress?.let { address ->
                                chainExplorer.getTokenUrl(explorerName, address)
                                    ?.let { url -> BlockExplorerLink(explorerName, url) }
                            },
                            tokenIdExplorerLink = nftAsset.contractAddress?.let { address ->
                                chainExplorer.getNftUrl(explorerName, address, nftAsset.tokenId)
                                    ?.let { url -> BlockExplorerLink(explorerName, url) }
                            },
                        )
                    }
            }
    }
}
