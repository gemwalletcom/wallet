package com.gemwallet.android.features.assets.viewmodels

import android.content.Context
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.AssetsQuery
import com.gemwallet.android.data.services.store.queries.NFTQuery
import com.gemwallet.android.data.services.store.queries.PerpetualsQuery
import com.gemwallet.android.data.services.store.queries.RecentActivityQuery
import com.gemwallet.android.data.services.store.queries.WalletSearchQuery
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualDataAggregate
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualSections
import com.gemwallet.android.domains.perpetual.aggregates.marketSections
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.asset_select.viewmodels.BaseAssetSelectViewModel
import com.gemwallet.android.features.asset_select.viewmodels.models.BaseSelectSearch
import com.gemwallet.android.features.assets.viewmodels.models.AssetListRowUIModel
import com.gemwallet.android.features.assets.viewmodels.models.uiModel
import com.gemwallet.android.ui.components.screen.assetPinnedToast
import com.gemwallet.android.ui.models.NftItemUIModel
import com.gemwallet.android.ui.models.toUIModels
import com.wallet.core.primitives.NFTData
import com.wallet.core.primitives.PerpetualId
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemAssetSelectionServiceInterface
import uniffi.gemstone.GemSearchScope
import uniffi.gemstone.GemSelectAssetState
import uniffi.gemstone.GemSelectAssetType
import uniffi.gemstone.GemWalletSearchCounts
import uniffi.gemstone.GemWalletSearchInput
import uniffi.gemstone.GemWalletSearchView
import uniffi.gemstone.perpetualMarketQuery
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class WalletSearchViewModel @Inject constructor(
    getSession: GetSession,
    assetsQuery: AssetsQuery,
    walletSearchQuery: WalletSearchQuery,
    recentActivityQuery: RecentActivityQuery,
    service: GemAssetSelectionServiceInterface,
    perpetualsQuery: PerpetualsQuery,
    nftQuery: NFTQuery,
    @IoDispatcher ioDispatcher: CoroutineDispatcher,
    @ApplicationContext context: Context,
) : BaseAssetSelectViewModel(
    getSession,
    recentActivityQuery,
    service,
    BaseSelectSearch(assetsQuery),
    GemSelectAssetType.WalletSearch,
    ioDispatcher,
    context,
) {

    override suspend fun searchRemote(query: String) {
        service.search(query, GemSearchScope.All)
    }

    private val perpetualSections: StateFlow<PerpetualSections> = currentQuery
        .flatMapLatest { query -> perpetualMarketQuery(query).let { perpetualsQuery(it.search, it.limit.toInt(), it.requiresVolume) } }
        .map { it.marketSections() }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, PerpetualSections())

    private val nftData: Flow<List<NFTData>> = getSession()
        .filterNotNull()
        .distinctUntilChangedBy { it.wallet.id }
        .flatMapLatest { nftQuery(it.wallet.id.id) }
        .map { data -> data.filter { it.assets.isNotEmpty() } }
        .flowOn(ioDispatcher)

    private val nfts: StateFlow<List<NftItemUIModel>> = combine(
        nftData,
        currentQuery,
    ) { data, query ->
        if (query.isEmpty()) emptyList() else searchNfts(data, query)
    }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val lists: StateFlow<List<AssetListRowUIModel>> = currentQuery
        .flatMapLatest { query -> walletSearchQuery.lists(query) }
        .map { lists -> lists.map { it.uiModel() } }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val assetCounts: Flow<Pair<Int, Int>> = combine(pinned, unpinned) { pinnedAssets, assets -> pinnedAssets.size to assets.size }

    private val searchCounts: Flow<GemWalletSearchCounts> = combine(
        recent,
        assetCounts,
        perpetualSections,
        lists,
        nfts,
    ) { recents, assets, perpetuals, lists, nfts ->
        GemWalletSearchCounts(
            recents = recents.size.toUInt(),
            pinnedAssets = assets.first.toUInt(),
            assets = assets.second.toUInt(),
            pinnedPerpetuals = perpetuals.pinned.size.toUInt(),
            perpetuals = perpetuals.markets.size.toUInt(),
            lists = lists.size.toUInt(),
            nfts = nfts.size.toUInt(),
        )
    }

    private val view: StateFlow<GemWalletSearchView?> = combine(getSession(), currentQuery, uiState, searchCounts) { session, query, base, counts ->
        session?.wallet?.let { wallet ->
            service.walletSearchView(GemWalletSearchInput(wallet.toGem(), query, base == GemSelectAssetState.LOADING, counts))
        }
    }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val state: StateFlow<GemSelectAssetState> = view
        .map { it?.state?.phase ?: GemSelectAssetState.IDLE }
        .stateIn(viewModelScope, SharingStarted.Eagerly, GemSelectAssetState.IDLE)

    val previewAssets: StateFlow<List<AssetInfoDataAggregate>> = combine(unpinned, view) { items, view ->
        items.take(view?.limits?.assets?.toInt() ?: 0)
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val hasMoreAssets: StateFlow<Boolean> = view
        .map { it?.hasMoreAssets == true }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val pinnedPerpetuals: StateFlow<List<PerpetualDataAggregate>> = combine(perpetualSections, view) { sections, view ->
        if (view?.state?.showsPinnedPerpetuals == true) sections.pinned else emptyList()
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val previewPerpetuals: StateFlow<List<PerpetualDataAggregate>> = combine(perpetualSections, view) { sections, view ->
        if (view?.state?.showsPerpetuals == true) sections.markets.take(view.limits.perpetuals.toInt()) else emptyList()
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val hasMorePerpetuals: StateFlow<Boolean> = view
        .map { it?.hasMorePerpetuals == true }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val previewNfts: StateFlow<List<NftItemUIModel>> = combine(nfts, view) { items, view ->
        items.take(view?.limits?.nfts?.toInt() ?: 0)
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val hasMoreNfts: StateFlow<Boolean> = view
        .map { it?.hasMoreNfts == true }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    private fun searchNfts(data: List<NFTData>, query: String): List<NftItemUIModel> = service.searchCollections(data.map { it.toGem() }, query).toUIModels()

    override fun assetsSearchLimit(query: String): Int = service.walletSearchLimits(query).fetch.toInt()

    fun onTogglePerpetualPin(perpetualId: PerpetualId) = viewModelScope.launch {
        val item = perpetualSections.value.let { it.pinned + it.markets }.firstOrNull { it.id == perpetualId } ?: return@launch
        if (setPerpetualPinned(perpetualId, !item.isPinned).isSuccess) {
            emitToast(assetPinnedToast(context, item.title, !item.isPinned))
        }
    }
}
