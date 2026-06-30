package com.gemwallet.android.features.assets.viewmodels

import android.content.Context
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.asset_select.coordinators.GetRecentAssets
import com.gemwallet.android.application.asset_select.coordinators.SearchListAssets
import com.gemwallet.android.application.asset_select.coordinators.SearchSelectAssets
import com.gemwallet.android.application.asset_select.coordinators.SwitchAssetVisibility
import com.gemwallet.android.application.asset_select.coordinators.ToggleAssetPin
import com.gemwallet.android.application.asset_select.coordinators.UpdateRecentAsset
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetuals
import com.gemwallet.android.application.perpetual.coordinators.TogglePerpetualPin
import com.gemwallet.android.application.session.coordinators.GetSession
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.cases.tokens.WalletSearchScopeCase
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.config.showPerpetuals
import com.gemwallet.android.data.repositories.tokens.WalletSearch
import com.gemwallet.android.data.repositories.tokens.listPriorityQuery
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualDataAggregate
import com.gemwallet.android.domains.search.WalletSearchConfig
import com.gemwallet.android.domains.search.WalletSearchTag
import com.gemwallet.android.domains.search.walletSearchTagOf
import com.gemwallet.android.features.asset_select.viewmodels.BaseAssetSelectViewModel
import com.gemwallet.android.features.asset_select.viewmodels.models.BaseSelectSearch
import com.gemwallet.android.features.asset_select.viewmodels.models.ListSelectSearch
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectSearch
import com.gemwallet.android.features.asset_select.viewmodels.models.UIState
import com.gemwallet.android.model.RecentType
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.AssetItemUIModel
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetTag
import com.wallet.core.primitives.PerpetualId
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AssetsResultsViewModel @Inject constructor(
    private val getSession: GetSession,
    searchSelectAssets: SearchSelectAssets,
    searchListAssets: SearchListAssets,
    getRecentAssets: GetRecentAssets,
    updateRecentAsset: UpdateRecentAsset,
    switchAssetVisibility: SwitchAssetVisibility,
    toggleAssetPin: ToggleAssetPin,
    @WalletSearch searchTokensCase: SearchTokensCase,
    private val searchScopeCase: WalletSearchScopeCase,
    getPerpetuals: GetPerpetuals,
    userConfig: UserConfig,
    private val togglePerpetualPin: TogglePerpetualPin,
    @ApplicationContext context: Context,
    savedStateHandle: SavedStateHandle,
) : BaseAssetSelectViewModel(
    getSession,
    getRecentAssets,
    updateRecentAsset,
    switchAssetVisibility,
    toggleAssetPin,
    searchTokensCase,
    resolveSearch(savedStateHandle, searchSelectAssets, searchListAssets),
    remoteSearch = false,
) {

    private val scope: WalletSearchTag = walletSearchTagOf(savedStateHandle.get<String?>(RouteArgument.Scope.key))
    val title: String = savedStateHandle.get<String?>(RouteArgument.Title.key)
        ?: context.getString(R.string.assets_title)

    private val isFetching = MutableStateFlow(true)
    private val isPullRefreshing = MutableStateFlow(false)
    val refreshing: StateFlow<Boolean> = isPullRefreshing

    val cappedAssets: StateFlow<List<AssetItemUIModel>> = combine(pinned, unpinned) { pinned, unpinned ->
        unpinned.take((WalletSearchConfig.resultsLimit - pinned.size).coerceAtLeast(0))
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val previewPerpetuals: StateFlow<List<PerpetualDataAggregate>> = when (scope) {
        is WalletSearchTag.List ->
            combine(
                getPerpetuals.getPerpetuals(listPriorityQuery(scope.id)),
                userConfig.showPerpetuals(getSession()),
            ) { items, show ->
                if (show) items.take(WalletSearchConfig.resultsLimit) else emptyList()
            }
                .flowOn(Dispatchers.IO)
                .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

        WalletSearchTag.All, is WalletSearchTag.Filter ->
            MutableStateFlow(emptyList<PerpetualDataAggregate>())
    }

    val state: StateFlow<UIState> = combine(
        pinned, cappedAssets, previewPerpetuals, isFetching,
    ) { pinned, assets, perpetuals, fetching ->
        when {
            pinned.isNotEmpty() || assets.isNotEmpty() || perpetuals.isNotEmpty() -> UIState.Idle
            fetching -> UIState.Loading
            else -> UIState.Empty
        }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, UIState.Loading)

    init {
        when (scope) {
            is WalletSearchTag.Filter -> selectedTag.value = scope.tag
            WalletSearchTag.All, is WalletSearchTag.List -> Unit
        }
        queryState.setTextAndPlaceCursorAtEnd(savedStateHandle.get<String?>(RouteArgument.Query.key).orEmpty())
        fetch(pull = false)
    }

    override fun assetsSearchLimit(query: String, tag: AssetTag?): Int = WalletSearchConfig.resultsLimit

    fun refresh() = fetch(pull = true)

    private fun fetch(pull: Boolean) {
        viewModelScope.launch(Dispatchers.IO) {
            isFetching.value = true
            if (pull) isPullRefreshing.value = true
            try {
                val session = getSession().filterNotNull().first()
                val chains = walletSearchChains(session.wallet)
                searchScopeCase.search(queryState.text.toString(), session.currency, chains, scope)
            } finally {
                isFetching.value = false
                isPullRefreshing.value = false
            }
        }
    }

    fun onTogglePerpetualPin(perpetualId: PerpetualId) {
        togglePerpetualPin.togglePin(perpetualId)
    }

    fun onOpenPerpetual(assetId: AssetId) {
        updateRecent(assetId, RecentType.Perpetual)
    }
}

private fun resolveSearch(
    savedStateHandle: SavedStateHandle,
    searchSelectAssets: SearchSelectAssets,
    searchListAssets: SearchListAssets,
): SelectSearch {
    return when (val scope = walletSearchTagOf(savedStateHandle.get<String?>(RouteArgument.Scope.key))) {
        is WalletSearchTag.List -> ListSelectSearch(searchListAssets, scope.id)
        WalletSearchTag.All, is WalletSearchTag.Filter -> BaseSelectSearch(searchSelectAssets)
    }
}
