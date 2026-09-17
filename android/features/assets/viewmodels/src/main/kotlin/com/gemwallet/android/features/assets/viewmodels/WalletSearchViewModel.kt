package com.gemwallet.android.features.assets.viewmodels

import android.content.Context
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.nft.cases.GetNftCollections
import com.gemwallet.android.application.perpetual.cases.GetPerpetuals
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.assets.AssetsSearchService
import com.gemwallet.android.data.services.gemstone.assets.RecentAssetsService
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualDataAggregate
import com.gemwallet.android.ext.chainIds
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.asset_select.viewmodels.BaseAssetSelectViewModel
import com.gemwallet.android.features.asset_select.viewmodels.models.BaseSelectSearch
import com.gemwallet.android.features.asset_select.viewmodels.models.UIState
import com.gemwallet.android.features.assets.viewmodels.models.AssetListRowUIModel
import com.gemwallet.android.features.assets.viewmodels.models.uiModel
import com.gemwallet.android.ui.components.screen.assetPinnedToast
import com.gemwallet.android.ui.models.NftItemUIModel
import com.gemwallet.android.ui.models.toUIModels
import com.wallet.core.primitives.NFTData
import com.wallet.core.primitives.PerpetualId
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemAssetSelectionServiceInterface
import uniffi.gemstone.GemSearchScope
import uniffi.gemstone.GemSelectAssetType
import uniffi.gemstone.GemWalletSearchCounts
import uniffi.gemstone.GemWalletSearchPhase
import uniffi.gemstone.walletSearchPhase

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class WalletSearchViewModel @Inject constructor(
    getSession: GetSession,
    searchService: AssetsSearchService,
    recentAssetsService: RecentAssetsService,
    service: GemAssetSelectionServiceInterface,
    getPerpetuals: GetPerpetuals,
    getNftCollections: GetNftCollections,
    @ApplicationContext context: Context,
) : BaseAssetSelectViewModel(
    getSession,
    recentAssetsService,
    service,
    BaseSelectSearch(searchService),
    GemSelectAssetType.WalletSearch,
    context,
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

    val hasMorePerpetuals: StateFlow<Boolean> = perpetuals
        .map { items -> limits().hasMorePerpetuals(items.size.toUInt()) }
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
        .map { items -> limits().hasMoreNfts(items.size.toUInt()) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val lists: StateFlow<List<AssetListRowUIModel>> = currentQuery
        .flatMapLatest { query -> searchService.searchLists(query) }
        .map { lists -> lists.map { it.uiModel() } }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val previewAssets: StateFlow<List<AssetInfoDataAggregate>> = combine(
        unpinned, currentQuery,
    ) { items, query ->
        items.take(assetsLimit(query))
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val hasMoreAssets: StateFlow<Boolean> = combine(
        unpinned, currentQuery,
    ) { unpinned, query ->
        limits(query).hasMoreAssets(unpinned.size.toUInt())
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val state: StateFlow<UIState> = combine(
        uiState, previewPerpetuals, pinnedPerpetuals, previewNfts,
    ) { base, preview, pinnedPerps, nfts ->
        val counts = GemWalletSearchCounts(
            recents = 0u,
            pinned = pinnedPerps.size.toUInt(),
            assets = 0u,
            perpetuals = preview.size.toUInt(),
            lists = 0u,
            nfts = nfts.size.toUInt(),
        )
        when (walletSearchPhase(counts, base is UIState.Loading)) {
            GemWalletSearchPhase.RESULTS -> UIState.Idle
            GemWalletSearchPhase.LOADING, GemWalletSearchPhase.EMPTY -> base
        }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, UIState.Idle)

    private fun limits(query: String = queryState.text.toString()) = service.walletSearchLimits(query)

    private fun assetsLimit(query: String): Int = limits(query).assets.toInt()

    private fun searchNfts(data: List<NFTData>, query: String): List<NftItemUIModel> =
        service.searchCollections(data.map { it.toGem() }, query).toUIModels()

    override fun assetsSearchLimit(query: String): Int = limits(query).fetch.toInt()

    fun onTogglePerpetualPin(perpetualId: PerpetualId) = viewModelScope.launch {
        val item = visiblePerpetuals.value.firstOrNull { it.id == perpetualId } ?: return@launch
        setPerpetualPinned(perpetualId, !item.isPinned)
        emitToast(assetPinnedToast(context, item.title, !item.isPinned))
    }

}
