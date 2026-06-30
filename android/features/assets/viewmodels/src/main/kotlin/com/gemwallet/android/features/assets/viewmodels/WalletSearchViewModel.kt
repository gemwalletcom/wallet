package com.gemwallet.android.features.assets.viewmodels

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.asset_select.coordinators.GetRecentAssets
import com.gemwallet.android.application.asset_select.coordinators.SearchSelectAssets
import com.gemwallet.android.application.asset_select.coordinators.SwitchAssetVisibility
import com.gemwallet.android.application.asset_select.coordinators.ToggleAssetPin
import com.gemwallet.android.application.asset_select.coordinators.UpdateRecentAsset
import com.gemwallet.android.application.assets.coordinators.GetSearchLists
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetuals
import com.gemwallet.android.application.perpetual.coordinators.TogglePerpetualPin
import com.gemwallet.android.application.session.coordinators.GetSession
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.config.showPerpetuals
import com.gemwallet.android.data.repositories.tokens.WalletSearch
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualDataAggregate
import com.gemwallet.android.domains.search.WalletSearchConfig
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.asset_select.viewmodels.BaseAssetSelectViewModel
import com.gemwallet.android.features.asset_select.viewmodels.models.BaseSelectSearch
import com.gemwallet.android.features.asset_select.viewmodels.models.UIState
import com.gemwallet.android.model.RecentAssetsRequest
import com.gemwallet.android.model.RecentType
import com.gemwallet.android.ui.components.list_item.AssetItemUIModel
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetList
import com.wallet.core.primitives.AssetTag
import com.wallet.core.primitives.PerpetualId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
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
    updateRecentAsset: UpdateRecentAsset,
    switchAssetVisibility: SwitchAssetVisibility,
    toggleAssetPin: ToggleAssetPin,
    @WalletSearch searchTokensCase: SearchTokensCase,
    getPerpetuals: GetPerpetuals,
    getSearchLists: GetSearchLists,
    userConfig: UserConfig,
    private val togglePerpetualPin: TogglePerpetualPin,
) : BaseAssetSelectViewModel(
    getSession,
    getRecentAssets,
    updateRecentAsset,
    switchAssetVisibility,
    toggleAssetPin,
    searchTokensCase,
    BaseSelectSearch(searchSelectAssets),
) {

    private val showPerpetuals = userConfig.showPerpetuals(getSession())

    private val visiblePerpetuals = combine(
        getPerpetuals.getPerpetuals(currentQuery.map { it.takeIf(String::isNotEmpty) }),
        showPerpetuals,
        selectedTag,
    ) { items, show, tag ->
        if (show && tag == null) items else emptyList()
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
        getRecentAssets(RecentAssetsRequest(types = listOf(RecentType.Perpetual)))
            .map { items -> items.mapTo(HashSet()) { it.asset.id.toIdentifier() } }
            .flowOn(Dispatchers.IO)
            .stateIn(viewModelScope, SharingStarted.Eagerly, emptySet())

    val lists: StateFlow<List<AssetList>> = currentQuery
        .flatMapLatest { query -> getSearchLists.getSearchLists(query) }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val previewAssets: StateFlow<List<AssetItemUIModel>> = combine(
        unpinned, currentQuery, selectedTag,
    ) { items, query, tag ->
        items.take(assetsLimit(query, tag))
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val hasMoreAssets: StateFlow<Boolean> = combine(
        pinned, unpinned, currentQuery, selectedTag,
    ) { pinned, unpinned, query, tag ->
        (pinned.size + unpinned.size) > assetsLimit(query, tag)
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val state: StateFlow<UIState> = combine(
        uiState, previewPerpetuals, pinnedPerpetuals,
    ) { base, preview, pinnedPerps ->
        if (preview.isNotEmpty() || pinnedPerps.isNotEmpty()) UIState.Idle else base
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, UIState.Idle)

    private fun assetsLimit(query: String, tag: AssetTag?): Int = when {
        query.isNotEmpty() -> WalletSearchConfig.assetsSearchLimit
        tag != null -> WalletSearchConfig.assetsTagLimit
        else -> WalletSearchConfig.assetsInitialLimit
    }

    override fun assetsSearchLimit(query: String, tag: AssetTag?): Int = assetsLimit(query, tag) + 1

    fun onPinAsset(assetId: AssetId) {
        val willPin = (pinned.value + unpinned.value).firstOrNull { it.asset.id == assetId }?.metadata?.isPinned != true
        onTogglePin(assetId)
        if (willPin) onChangeVisibility(assetId, true)
    }

    fun onTogglePerpetualPin(perpetualId: PerpetualId) {
        togglePerpetualPin.togglePin(perpetualId)
    }

    fun onOpenPerpetual(assetId: AssetId) {
        updateRecent(assetId, RecentType.Perpetual)
    }
}
