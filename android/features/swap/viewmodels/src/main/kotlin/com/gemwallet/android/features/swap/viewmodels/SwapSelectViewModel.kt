package com.gemwallet.android.features.swap.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.assets.AssetsSearchService
import com.gemwallet.android.data.services.gemstone.assets.RecentAssetsService
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.features.asset_select.viewmodels.BaseAssetSelectViewModel
import com.gemwallet.android.features.asset_select.viewmodels.models.BaseSelectSearch
import com.gemwallet.android.ui.models.navigation.RouteArgument
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemAssetSelectionServiceInterface
import uniffi.gemstone.GemSelectAssetType
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class SwapSelectViewModel @Inject constructor(
    getSession: GetSession,
    searchService: AssetsSearchService,
    recentAssetsService: RecentAssetsService,
    service: GemAssetSelectionServiceInterface,
    savedStateHandle: SavedStateHandle,
) : BaseAssetSelectViewModel(
    getSession = getSession,
    recentAssetsService = recentAssetsService,
    service = service,
    search = BaseSelectSearch(searchService),
    selectType = when (savedStateHandle.requireSwapItemType()) {
        SwapItemType.Pay -> GemSelectAssetType.SwapPay
        SwapItemType.Receive -> GemSelectAssetType.SwapReceive(payAssetId = savedStateHandle.get<String?>(RouteArgument.FromAssetId.key))
    },
) {

    val payAssetId = savedStateHandle.getStateFlow<String?>(RouteArgument.FromAssetId.key, null)
        .mapLatest { it?.toAssetId() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)
    val receiveAssetId = savedStateHandle.getStateFlow<String?>(RouteArgument.ToAssetId.key, null)
        .mapLatest { it?.toAssetId() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)
    val select = MutableStateFlow(savedStateHandle.requireSwapItemType())
}

private fun SavedStateHandle.requireSwapItemType(): SwapItemType =
    checkNotNull(get<SwapItemType>(RouteArgument.SwapItemType.key)) {
        "Missing route argument: ${RouteArgument.SwapItemType.key}"
    }
