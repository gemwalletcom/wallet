package com.gemwallet.android.features.assets.viewmodels

import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.asset_select.coordinators.GetRecentAssets
import com.gemwallet.android.application.asset_select.coordinators.SearchSelectAssets
import com.gemwallet.android.application.asset_select.coordinators.SwitchAssetVisibility
import com.gemwallet.android.application.asset_select.coordinators.ToggleAssetPin
import com.gemwallet.android.application.asset_select.coordinators.UpdateRecentAsset
import com.gemwallet.android.application.session.coordinators.GetSession
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.domains.search.WalletSearchConfig
import com.gemwallet.android.features.asset_select.viewmodels.BaseAssetSelectViewModel
import com.gemwallet.android.features.asset_select.viewmodels.models.BaseSelectSearch
import com.gemwallet.android.ui.components.list_item.AssetItemUIModel
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.AssetTag
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AssetsResultsViewModel @Inject constructor(
    getSession: GetSession,
    searchSelectAssets: SearchSelectAssets,
    getRecentAssets: GetRecentAssets,
    updateRecentAsset: UpdateRecentAsset,
    switchAssetVisibility: SwitchAssetVisibility,
    toggleAssetPin: ToggleAssetPin,
    searchTokensCase: SearchTokensCase,
    savedStateHandle: SavedStateHandle,
) : BaseAssetSelectViewModel(
    getSession,
    getRecentAssets,
    updateRecentAsset,
    switchAssetVisibility,
    toggleAssetPin,
    searchTokensCase,
    BaseSelectSearch(searchSelectAssets),
    remoteSearch = false,
) {

    init {
        val tag = savedStateHandle.get<String?>(RouteArgument.Tag.key)
            ?.let { value -> AssetTag.entries.firstOrNull { it.string == value } }
        selectedTag.value = tag
        queryState.setTextAndPlaceCursorAtEnd(savedStateHandle.get<String?>(RouteArgument.Query.key).orEmpty())
    }

    override fun assetsSearchLimit(query: String, tag: AssetTag?): Int = WalletSearchConfig.resultsLimit

    val cappedAssets: StateFlow<List<AssetItemUIModel>> = combine(pinned, unpinned) { pinned, unpinned ->
        unpinned.take((WalletSearchConfig.resultsLimit - pinned.size).coerceAtLeast(0))
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())
}
