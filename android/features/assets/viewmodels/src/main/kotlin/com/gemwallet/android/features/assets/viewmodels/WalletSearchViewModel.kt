package com.gemwallet.android.features.assets.viewmodels

import com.wallet.core.primitives.Wallet
import com.gemwallet.android.serializer.toJson
import uniffi.gemstone.GemSearchScope
import uniffi.gemstone.GemAssetSelectionServiceInterface
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.asset_select.cases.GetRecentAssets
import com.gemwallet.android.application.asset_select.cases.SearchSelectAssets
import com.gemwallet.android.application.assets.cases.GetSearchLists
import com.gemwallet.android.application.nft.cases.GetNftCollections
import com.gemwallet.android.application.perpetual.cases.GetPerpetuals
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.data.services.gemstone.config.showPerpetuals
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualDataAggregate
import com.gemwallet.android.domains.search.WalletSearchConfig
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.asset_select.viewmodels.BaseAssetSelectViewModel
import com.gemwallet.android.features.asset_select.viewmodels.models.BaseSelectSearch
import com.gemwallet.android.features.asset_select.viewmodels.models.UIState
import com.gemwallet.android.model.RecentAssetsRequest
import com.wallet.core.primitives.RecentActivityType
import com.gemwallet.android.ui.models.AssetToast
import com.gemwallet.android.ui.models.NftItemUIModel
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetList
import com.wallet.core.primitives.NFTData
import com.wallet.core.primitives.PerpetualId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class WalletSearchViewModel @Inject constructor(
    getSession: GetSession,
    searchSelectAssets: SearchSelectAssets,
    getRecentAssets: GetRecentAssets,
    service: GemAssetSelectionServiceInterface,
    getPerpetuals: GetPerpetuals,
    getNftCollections: GetNftCollections,
    getSearchLists: GetSearchLists,
    userConfig: UserConfig,
) : BaseAssetSelectViewModel(
    getSession,
    getRecentAssets,
    service,
    BaseSelectSearch(searchSelectAssets),
) {

    override suspend fun searchRemote(wallet: Wallet, query: String) {
        service.search(wallet.toJson(), query, GemSearchScope.All)
    }

    private val showPerpetuals = userConfig.showPerpetuals(getSession())

    private val visiblePerpetuals = combine(
        getPerpetuals.getPerpetuals(currentQuery.map { it.takeIf(String::isNotEmpty) }),
        showPerpetuals,
    ) { items, show ->
        if (show) items else emptyList()
    }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val pinnedPerpetuals: StateFlow<List<PerpetualDataAggregate>> = visiblePerpetuals
        .map { items -> items.filter { it.isPinned } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val perpetuals: StateFlow<List<PerpetualDataAggregate>> = visiblePerpetuals
        .map { items -> items.filter { !it.isPinned } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val previewPerpetuals: StateFlow<List<PerpetualDataAggregate>> = perpetuals
        .map { items -> items.take(WalletSearchConfig.perpetualsPreviewLimit) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val hasMorePerpetuals: StateFlow<Boolean> = visiblePerpetuals
        .map { items -> items.size > WalletSearchConfig.perpetualsPreviewLimit }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val perpetualRecentIds: StateFlow<Set<String>> =
        getRecentAssets(RecentAssetsRequest(types = listOf(RecentActivityType.Perpetual)))
            .map { items -> items.mapTo(HashSet()) { it.asset.id.toIdentifier() } }
            .flowOn(Dispatchers.IO)
            .stateIn(viewModelScope, SharingStarted.Eagerly, emptySet())

    private val nftData: Flow<List<NFTData>> = getNftCollections(null)
        .map { data -> data.filter { it.assets.isNotEmpty() } }
        .flowOn(Dispatchers.IO)

    private val nfts: StateFlow<List<NftItemUIModel>> = combine(
        nftData, currentQuery,
    ) { data, query ->
        if (query.isEmpty()) emptyList() else searchNfts(data, query)
    }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val previewNfts: StateFlow<List<NftItemUIModel>> = nfts
        .map { items -> items.take(WalletSearchConfig.nftsPreviewLimit) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val hasMoreNfts: StateFlow<Boolean> = nfts
        .map { items -> items.size > WalletSearchConfig.nftsPreviewLimit }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val lists: StateFlow<List<AssetList>> = currentQuery
        .flatMapLatest { query -> getSearchLists.getSearchLists(query) }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val previewAssets: StateFlow<List<AssetInfoDataAggregate>> = combine(
        unpinned, currentQuery,
    ) { items, query ->
        items.take(assetsLimit(query))
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val hasMoreAssets: StateFlow<Boolean> = combine(
        pinned, unpinned, currentQuery,
    ) { pinned, unpinned, query ->
        (pinned.size + unpinned.size) > assetsLimit(query)
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val state: StateFlow<UIState> = combine(
        uiState, previewPerpetuals, pinnedPerpetuals, previewNfts,
    ) { base, preview, pinnedPerps, nfts ->
        if (preview.isNotEmpty() || pinnedPerps.isNotEmpty() || nfts.isNotEmpty()) UIState.Idle else base
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, UIState.Idle)

    private fun assetsLimit(query: String): Int = if (query.isNotEmpty()) {
        WalletSearchConfig.assetsSearchLimit
    } else {
        WalletSearchConfig.assetsInitialLimit
    }

    private fun searchNfts(data: List<NFTData>, query: String): List<NftItemUIModel> = data
        .sortedWith(compareByDescending<NFTData> { it.assets.size }.thenBy { it.collection.name })
        .flatMap { nft ->
            if (nft.collection.name.contains(query, ignoreCase = true)) {
                listOf(nft.toNftItem())
            } else {
                nft.assets
                    .filter { it.name.contains(query, ignoreCase = true) }
                    .sortedBy { it.name }
                    .map { NftItemUIModel(nft.collection, it) }
            }
        }

    private fun NFTData.toNftItem(): NftItemUIModel =
        if (assets.size == 1) NftItemUIModel(collection, assets.first()) else NftItemUIModel(collection, null, assets.size)

    override fun assetsSearchLimit(query: String): Int = assetsLimit(query) + 1

    fun onPinAsset(assetId: AssetId) {
        val willPin = (pinned.value + unpinned.value).firstOrNull { it.asset.id == assetId }?.pinned != true
        onTogglePin(assetId)
        if (willPin) onChangeVisibility(assetId, true)
    }

    fun onTogglePerpetualPin(perpetualId: PerpetualId) = viewModelScope.launch {
        val item = visiblePerpetuals.value.firstOrNull { it.id == perpetualId } ?: return@launch
        setPerpetualPinned(perpetualId, !item.isPinned)
        emitToast(AssetToast.Pin(item.name, !item.isPinned))
    }

    fun onOpenPerpetual(assetId: AssetId) {
        updateRecent(assetId, RecentActivityType.Perpetual)
    }
}
