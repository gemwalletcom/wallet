package com.gemwallet.android.features.nft.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.cases.nft.GetAssetNft
import com.gemwallet.android.cases.nodes.GetCurrentBlockExplorer
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.getAccount
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.NFTAttribute
import com.wallet.core.primitives.NFTCollection
import uniffi.gemstone.Explorer
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import javax.inject.Inject

val nftAssetIdArg = "assetId"

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class NftDetailsViewModel @Inject constructor(
    sessionRepository: SessionRepository,
    private val getAssetNft: GetAssetNft,
    private val getCurrentBlockExplorer: GetCurrentBlockExplorer,
    savedStateHandle: SavedStateHandle
) : ViewModel() {

    private val assetId = savedStateHandle.getStateFlow<String?>(nftAssetIdArg, null)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)
    private val session = sessionRepository.session()

    val nftAsset = combine(
        session,
        assetId.filterNotNull()
    ) { session, assetId -> Pair(session, assetId) }
    .flatMapLatest {
        val (session, assetId) = it
        getAssetNft.getAssetNft(it.second)
            .filterNotNull()
            .map {
                val nftAsset = it.assets.first()
                val chain = nftAsset.chain
                val explorerName = getCurrentBlockExplorer.getCurrentBlockExplorer(chain)
                val chainExplorer = Explorer(chain.string)
                NftAssetDetailsUIModel(
                    collection = it.collection,
                    asset = nftAsset,
                    account = session?.wallet?.getAccount(chain)!!,
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
    .catch { }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)
}

class NftAssetDetailsUIModel(
    val collection: NFTCollection,
    val asset: NFTAsset,
    val account: Account,
    val contractExplorerLink: BlockExplorerLink? = null,
    val tokenIdExplorerLink: BlockExplorerLink? = null,
) {
    val imageUrl: String get() = asset.images.preview.url
    val assetName: String get() = asset.name
    val collectionName: String get() = collection.name
    val description: String? get() = asset.description
    val attributes: List<NFTAttribute> get() = asset.attributes
}