package com.gemwallet.android.features.wallet.viewmodels

import android.content.Context
import android.util.Log
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.AssetsQuery
import com.gemwallet.android.data.services.store.queries.PerpetualsQuery
import com.gemwallet.android.data.services.store.queries.RecentActivityQuery
import com.gemwallet.android.data.services.store.queries.WalletSearchQuery
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.search.WalletSearchTag
import com.gemwallet.android.domains.search.toGem
import com.gemwallet.android.domains.search.walletSearchTagOf
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.assets.viewmodels.select.BaseSelectAssetViewModel
import com.gemwallet.android.features.assets.viewmodels.select.models.BaseSelectSearch
import com.gemwallet.android.features.assets.viewmodels.select.models.ListSelectSearch
import com.gemwallet.android.features.assets.viewmodels.select.models.SelectSearch
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.PerpetualData
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemAssetSelectionServiceInterface
import uniffi.gemstone.GemPerpetualMarketItem
import uniffi.gemstone.GemSelectAssetState
import uniffi.gemstone.GemSelectAssetType
import uniffi.gemstone.GemWalletSearchResultsInput
import uniffi.gemstone.GemWalletSearchResultsView
import uniffi.gemstone.perpetualMarketQuery
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AssetsResultsViewModel @Inject constructor(
    private val getSession: GetSession,
    assetsQuery: AssetsQuery,
    walletSearchQuery: WalletSearchQuery,
    recentActivityQuery: RecentActivityQuery,
    service: GemAssetSelectionServiceInterface,
    perpetualsQuery: PerpetualsQuery,
    @IoDispatcher ioDispatcher: CoroutineDispatcher,
    @ApplicationContext context: Context,
    savedStateHandle: SavedStateHandle,
) : BaseSelectAssetViewModel(
    getSession,
    recentActivityQuery,
    service,
    selectSearchOf(savedStateHandle, assetsQuery, walletSearchQuery, service),
    GemSelectAssetType.WalletSearchResults,
    ioDispatcher,
    context,
) {

    private val scope: WalletSearchTag = walletSearchTagOf(savedStateHandle.get<String?>(RouteArgument.Scope.key))
    private val searchKey: String = searchKeyOf(savedStateHandle, service)
    val title: String = savedStateHandle.get<String?>(RouteArgument.Title.key)
        ?: context.getString(R.string.assets_title)

    private val isSearching = MutableStateFlow(true)
    private val isPullRefreshing = MutableStateFlow(false)
    val refreshing: StateFlow<Boolean> = isPullRefreshing

    private val perpetuals: Flow<List<PerpetualData>> = perpetualMarketQuery(searchKey).let { perpetualsQuery(it.search, it.limit.toInt(), it.requiresVolume) }

    private val content: StateFlow<SearchContent<GemWalletSearchResultsView>?> = combine(getSession(), isSearching, pinned, unpinned, perpetuals) { session, searching, pinned, unpinned, perpetuals ->
        session?.wallet?.let { wallet ->
            val assets = pinned + unpinned
            val input = GemWalletSearchResultsInput(
                wallet = wallet.toGem(),
                scope = scope.toGem(),
                isLoading = searching,
                assetIds = assets.map { it.asset.id.toIdentifier() },
                pinnedAssetIds = pinned.map { it.asset.id.toIdentifier() },
                perpetuals = perpetuals.map { it.toGem() },
            )
            SearchContent(service.walletSearchResultsView(input), assets)
        }
    }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val state: StateFlow<GemSelectAssetState> = content.select(viewModelScope, GemSelectAssetState.LOADING) { it.view.state.phase }
    val pinnedAssets: StateFlow<List<AssetInfoDataAggregate>> = content.select(viewModelScope, emptyList()) { it.assets(it.view.pinnedAssetIds) }
    val assets: StateFlow<List<AssetInfoDataAggregate>> = content.select(viewModelScope, emptyList()) { it.assets(it.view.assetIds) }
    val perpetualItems: StateFlow<List<GemPerpetualMarketItem>> = content.select(viewModelScope, emptyList()) { it.view.perpetuals }

    init {
        queryState.setTextAndPlaceCursorAtEnd(savedStateHandle.get<String?>(RouteArgument.Query.key).orEmpty())
        search(pull = false)
    }

    override fun assetsSearchLimit(query: String): Int = service.walletSearchLimits(query).results.toInt()

    fun refresh() = search(pull = true)

    private fun search(pull: Boolean) {
        viewModelScope.launch(ioDispatcher) {
            isSearching.value = true
            if (pull) isPullRefreshing.value = true
            try {
                runCatchingCancellable { service.search(queryState.text.toString(), scope.toGem()) }
                    .onFailure { Log.e("AssetsResults", "search failed", it) }
            } finally {
                isSearching.value = false
                isPullRefreshing.value = false
            }
        }
    }

}

private fun searchKeyOf(savedStateHandle: SavedStateHandle, service: GemAssetSelectionServiceInterface): String {
    val query = savedStateHandle.get<String?>(RouteArgument.Query.key).orEmpty()
    val scope = walletSearchTagOf(savedStateHandle.get<String?>(RouteArgument.Scope.key))
    return service.searchKey(query, scope.toGem())
}

private fun selectSearchOf(savedStateHandle: SavedStateHandle, assetsQuery: AssetsQuery, walletSearchQuery: WalletSearchQuery, service: GemAssetSelectionServiceInterface): SelectSearch =
    when (walletSearchTagOf(savedStateHandle.get<String?>(RouteArgument.Scope.key))) {
        is WalletSearchTag.List -> ListSelectSearch(walletSearchQuery, searchKeyOf(savedStateHandle, service))
        WalletSearchTag.All -> BaseSelectSearch(assetsQuery)
    }
