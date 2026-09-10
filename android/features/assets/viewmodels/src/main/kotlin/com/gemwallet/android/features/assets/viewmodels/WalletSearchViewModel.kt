package com.gemwallet.android.features.assets.viewmodels

import com.gemwallet.android.ext.chainIds
import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemSearchScope
import uniffi.gemstone.GemAssetSelectionServiceInterface
import uniffi.gemstone.GemSelectAssetType
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.nft.cases.GetNftCollections
import com.gemwallet.android.application.perpetual.cases.GetPerpetuals
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.assets.AssetsSearchService
import com.gemwallet.android.data.services.gemstone.assets.RecentAssetsService
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualDataAggregate
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.asset_select.viewmodels.BaseAssetSelectViewModel
import com.gemwallet.android.features.asset_select.viewmodels.models.BaseSelectSearch
import com.gemwallet.android.features.asset_select.viewmodels.models.UIState
import com.gemwallet.android.model.RecentAssetsRequest
import com.gemwallet.android.ui.models.AssetToast
import com.gemwallet.android.ui.models.NftItemUIModel
import com.gemwallet.android.ui.models.toUIModel
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.RecentActivityType
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
    searchService: AssetsSearchService,
    recentAssetsService: RecentAssetsService,
    service: GemAssetSelectionServiceInterface,
    getPerpetuals: GetPerpetuals,
    getNftCollections: GetNftCollections,
) : BaseAssetSelectViewModel(
    getSession,
    recentAssetsService,
    service,
    BaseSelectSearch(searchService),
    GemSelectAssetType.WalletSearch,
) {

    override suspend fun searchRemote(query: String) {
        service.search(query, GemSearchScope.All)
    }

    private val showPerpetuals = getSession().map { session -> session?.wallet?.let { service.showPerpetuals(it.type.toGem(), it.chainIds) } ?: false }

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
        .map { items -> items.take(limits().perpetuals.toInt()) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val hasMorePerpetuals: StateFlow<Boolean> = visiblePerpetuals
        .map { items -> items.size > limits().perpetuals.toInt() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

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
        .map { items -> items.take(limits().nfts.toInt()) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val hasMoreNfts: StateFlow<Boolean> = nfts
        .map { items -> items.size > limits().nfts.toInt() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val lists: StateFlow<List<AssetList>> = currentQuery
        .flatMapLatest { query -> searchService.searchLists(query) }
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

    private fun limits(query: String = queryState.text.toString()) = service.walletSearchLimits(query)

    private fun assetsLimit(query: String): Int = limits(query).assets.toInt()

    private fun searchNfts(data: List<NFTData>, query: String): List<NftItemUIModel> =
        service.searchCollections(data.map { it.toGem() }, query).map { it.toUIModel() }

    override fun assetsSearchLimit(query: String): Int = limits(query).fetch.toInt()

    fun onTogglePerpetualPin(perpetualId: PerpetualId) = viewModelScope.launch {
        val item = visiblePerpetuals.value.firstOrNull { it.id == perpetualId } ?: return@launch
        setPerpetualPinned(perpetualId, !item.isPinned)
        emitToast(AssetToast.Pin(item.name, !item.isPinned))
    }

}
