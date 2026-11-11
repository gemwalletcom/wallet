package com.gemwallet.android.features.wallet.viewmodels

import android.content.Context
import android.util.Log
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.AssetsQuery
import com.gemwallet.android.data.services.store.queries.RecentActivityQuery
import com.gemwallet.android.data.services.store.queries.WalletSearchQuery
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.assets.viewmodels.select.BaseSelectAssetViewModel
import com.gemwallet.android.features.assets.viewmodels.select.models.BaseSelectSearch
import com.gemwallet.android.features.assets.viewmodels.select.models.ListSelectSearch
import com.gemwallet.android.features.assets.viewmodels.select.models.SelectSearch
import com.gemwallet.android.serializer.unpackRoutePayload
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
import uniffi.gemstone.GemSearchScope
import uniffi.gemstone.GemSelectAssetState
import uniffi.gemstone.GemSelectAssetType
import uniffi.gemstone.GemWalletSearchResultsInput
import uniffi.gemstone.GemWalletSearchResultsView
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AssetsResultsViewModel @Inject constructor(
    private val getSession: GetSession,
    assetsQuery: AssetsQuery,
    walletSearchQuery: WalletSearchQuery,
    recentActivityQuery: RecentActivityQuery,
    service: GemAssetSelectionServiceInterface,
    @IoDispatcher ioDispatcher: CoroutineDispatcher,
    @ApplicationContext context: Context,
    savedStateHandle: SavedStateHandle,
) : BaseSelectAssetViewModel(
    getSession,
    recentActivityQuery,
    service,
    selectSearchOf(savedStateHandle, assetsQuery, walletSearchQuery),
    GemSelectAssetType.WalletSearchResults,
    ioDispatcher,
    context,
) {

    private val scope: GemSearchScope = searchScopeOf(savedStateHandle)
    private val searchKey: String = searchKeyOf(savedStateHandle)
    val title: String = savedStateHandle.get<String?>(RouteArgument.Title.key)
        ?: context.getString(R.string.assets_title)

    private val isSearching = MutableStateFlow(true)
    private val isPullRefreshing = MutableStateFlow(false)
    val refreshing: StateFlow<Boolean> = isPullRefreshing

    private val perpetuals: Flow<List<PerpetualData>> = walletSearchQuery.perpetuals(searchKey, scope, assetsSearchLimit(savedStateHandle.get<String?>(RouteArgument.Query.key).orEmpty()))

    private val content: StateFlow<SearchContent<GemWalletSearchResultsView>?> = combine(getSession(), isSearching, sections, perpetuals) { session, searching, sections, perpetuals ->
        session?.wallet?.let { wallet ->
            val assets = sections.pinned + sections.unpinned
            val input = GemWalletSearchResultsInput(
                wallet = wallet.toGem(),
                scope = scope,
                isLoading = searching,
                assetIds = assets.map { it.asset.id.toIdentifier() },
                pinnedAssetIds = sections.pinned.map { it.asset.id.toIdentifier() },
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

    override fun assetsSearchLimit(query: String): Int = service.walletSearchLimits(query).results

    fun refresh() = search(pull = true)

    private fun search(pull: Boolean) {
        viewModelScope.launch(ioDispatcher) {
            isSearching.value = true
            if (pull) isPullRefreshing.value = true
            try {
                runCatchingCancellable { service.search(queryState.text.toString(), scope) }
                    .onFailure { Log.e("AssetsResults", "search failed", it) }
            } finally {
                isSearching.value = false
                isPullRefreshing.value = false
            }
        }
    }
}

private fun searchScopeOf(savedStateHandle: SavedStateHandle): GemSearchScope = savedStateHandle.get<String?>(RouteArgument.Scope.key)?.let { unpackRoutePayload<GemSearchScope>(it) } ?: GemSearchScope.All

private fun searchKeyOf(savedStateHandle: SavedStateHandle): String {
    val query = savedStateHandle.get<String?>(RouteArgument.Query.key).orEmpty()
    return searchScopeOf(savedStateHandle).searchKey(query)
}

private fun selectSearchOf(savedStateHandle: SavedStateHandle, assetsQuery: AssetsQuery, walletSearchQuery: WalletSearchQuery): SelectSearch = when (searchScopeOf(savedStateHandle)) {
    is GemSearchScope.List -> ListSelectSearch(walletSearchQuery, searchKeyOf(savedStateHandle))
    GemSearchScope.All -> BaseSelectSearch(assetsQuery)
}
