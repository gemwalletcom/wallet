package com.gemwallet.android.features.assets.viewmodels

import android.content.Context
import android.util.Log
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.perpetual.cases.GetPerpetuals
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.assets.AssetsSearchService
import com.gemwallet.android.data.services.gemstone.assets.RecentAssetsService
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualDataAggregate
import com.gemwallet.android.domains.search.WalletSearchTag
import com.gemwallet.android.domains.search.toGem
import com.gemwallet.android.domains.search.walletSearchTagOf
import com.gemwallet.android.ext.chainIds
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.asset_select.viewmodels.BaseAssetSelectViewModel
import com.gemwallet.android.features.asset_select.viewmodels.models.BaseSelectSearch
import com.gemwallet.android.features.asset_select.viewmodels.models.ListSelectSearch
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectSearch
import com.gemwallet.android.features.asset_select.viewmodels.models.UIState
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.assetPinnedToast
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.PerpetualId
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemAssetSelectionServiceInterface
import uniffi.gemstone.GemSelectAssetType
import uniffi.gemstone.GemWalletSearchCounts
import uniffi.gemstone.GemWalletSearchPhase
import uniffi.gemstone.walletSearchState
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AssetsResultsViewModel @Inject constructor(
    private val getSession: GetSession,
    searchService: AssetsSearchService,
    recentAssetsService: RecentAssetsService,
    service: GemAssetSelectionServiceInterface,
    getPerpetuals: GetPerpetuals,
    @IoDispatcher ioDispatcher: CoroutineDispatcher,
    @ApplicationContext context: Context,
    savedStateHandle: SavedStateHandle,
) : BaseAssetSelectViewModel(
    getSession,
    recentAssetsService,
    service,
    selectSearchOf(savedStateHandle, searchService, service),
    GemSelectAssetType.WalletSearchResults,
    ioDispatcher,
    context,
) {

    private val scope: WalletSearchTag = walletSearchTagOf(savedStateHandle.get<String?>(RouteArgument.Scope.key))
    private val searchKey: String = searchKeyOf(savedStateHandle, service)
    val title: String = savedStateHandle.get<String?>(RouteArgument.Title.key)
        ?: context.getString(R.string.assets_title)

    private val isFetching = MutableStateFlow(true)
    private val isPullRefreshing = MutableStateFlow(false)
    val refreshing: StateFlow<Boolean> = isPullRefreshing

    val previewPerpetuals: StateFlow<List<PerpetualDataAggregate>> = when (scope) {
        is WalletSearchTag.List ->
            combine(
                getPerpetuals.getPerpetuals(searchKey),
                getSession().map { session -> session?.wallet?.let { service.showPerpetuals(it.type.toGem(), it.chainIds) } ?: false },
            ) { items, show ->
                if (show) items.take(resultsLimit()) else emptyList()
            }
                .flowOn(ioDispatcher)
                .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

        WalletSearchTag.All ->
            MutableStateFlow(emptyList<PerpetualDataAggregate>())
    }

    val state: StateFlow<UIState> = combine(
        pinned,
        unpinned,
        previewPerpetuals,
        isFetching,
    ) { pinned, assets, perpetuals, fetching ->
        val counts = GemWalletSearchCounts(
            recents = 0u,
            pinnedAssets = pinned.size.toUInt(),
            assets = assets.size.toUInt(),
            pinnedPerpetuals = 0u,
            perpetuals = perpetuals.size.toUInt(),
            lists = 0u,
            nfts = 0u,
        )
        when (walletSearchState(counts, fetching).phase) {
            GemWalletSearchPhase.RESULTS -> UIState.Idle
            GemWalletSearchPhase.LOADING -> UIState.Loading
            GemWalletSearchPhase.EMPTY -> UIState.Empty
        }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, UIState.Loading)

    init {
        queryState.setTextAndPlaceCursorAtEnd(savedStateHandle.get<String?>(RouteArgument.Query.key).orEmpty())
        fetch(pull = false)
    }

    override fun assetsSearchLimit(query: String): Int = resultsLimit(query)

    private fun resultsLimit(query: String = queryState.text.toString()): Int = service.walletSearchLimits(query).results.toInt()

    fun refresh() = fetch(pull = true)

    private fun fetch(pull: Boolean) {
        viewModelScope.launch(ioDispatcher) {
            isFetching.value = true
            if (pull) isPullRefreshing.value = true
            try {
                runCatchingCancellable { service.search(queryState.text.toString(), scope.toGem()) }
                    .onFailure { Log.e("AssetsResults", "search failed", it) }
            } finally {
                isFetching.value = false
                isPullRefreshing.value = false
            }
        }
    }

    fun onTogglePerpetualPin(perpetualId: PerpetualId) = viewModelScope.launch {
        val item = previewPerpetuals.value.firstOrNull { it.id == perpetualId } ?: return@launch
        setPerpetualPinned(perpetualId, !item.isPinned)
        emitToast(assetPinnedToast(context, item.title, !item.isPinned))
    }
}

private fun searchKeyOf(savedStateHandle: SavedStateHandle, service: GemAssetSelectionServiceInterface): String {
    val query = savedStateHandle.get<String?>(RouteArgument.Query.key).orEmpty()
    val scope = walletSearchTagOf(savedStateHandle.get<String?>(RouteArgument.Scope.key))
    return service.searchKey(query, scope.toGem())
}

private fun selectSearchOf(savedStateHandle: SavedStateHandle, searchService: AssetsSearchService, service: GemAssetSelectionServiceInterface): SelectSearch =
    when (walletSearchTagOf(savedStateHandle.get<String?>(RouteArgument.Scope.key))) {
        is WalletSearchTag.List -> ListSelectSearch(searchService, searchKeyOf(savedStateHandle, service))
        WalletSearchTag.All -> BaseSelectSearch(searchService)
    }
