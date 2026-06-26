package com.gemwallet.android.features.swap.viewmodels

import androidx.compose.foundation.text.input.clearText
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.asset_select.coordinators.GetRecentAssets
import com.gemwallet.android.application.asset_select.coordinators.SwitchAssetVisibility
import com.gemwallet.android.application.asset_select.coordinators.ToggleAssetPin
import com.gemwallet.android.application.asset_select.coordinators.UpdateRecentAsset
import com.gemwallet.android.application.session.coordinators.GetSession
import com.gemwallet.android.application.swap.coordinators.SearchSwapAssets
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.features.asset_select.viewmodels.BaseAssetSelectViewModel
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectAssetFilters
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectSearch
import com.gemwallet.android.model.AssetFilter
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.RecentType
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.AssetId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class SwapSelectViewModel @Inject constructor(
    getSession: GetSession,
    getRecentAssets: GetRecentAssets,
    updateRecentAsset: UpdateRecentAsset,
    switchAssetVisibility: SwitchAssetVisibility,
    toggleAssetPin: ToggleAssetPin,
    searchTokensCase: SearchTokensCase,
    searchSwapAssets: SearchSwapAssets,
    savedStateHandle: SavedStateHandle,
) : BaseAssetSelectViewModel(
    getSession = getSession,
    getRecentAssets = getRecentAssets,
    updateRecentAsset = updateRecentAsset,
    switchAssetVisibility = switchAssetVisibility,
    toggleAssetPin = toggleAssetPin,
    searchTokensCase = searchTokensCase,
    search = SwapSelectSearch(searchSwapAssets),
    remoteSearch = savedStateHandle.requireSwapItemType() == SwapItemType.Receive,
) {

    val payAssetId = savedStateHandle.getStateFlow<String?>(RouteArgument.FromAssetId.key, null)
        .mapLatest { it?.toAssetId() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)
    val receiveAssetId = savedStateHandle.getStateFlow<String?>(RouteArgument.ToAssetId.key, null)
        .mapLatest { it?.toAssetId() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)
    val select = MutableStateFlow(savedStateHandle.requireSwapItemType())

    val state = combine(payAssetId, receiveAssetId, select) { pay, receive, select ->
        setPair(select, pay, receive)
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun setPair(type: SwapItemType, payId: AssetId?, receiveId: AssetId?) {
        queryState.clearText()
        (search as? SwapSelectSearch)?.apply {
            this.swapItemType.update { type }
            this.payId.update { payId }
            this.receiveId.update { receiveId }
        }
    }

    override fun assetFilters() = setOf(AssetFilter.Swappable)

    override val recentTypes: List<RecentType> get() = listOf(RecentType.SwapSelect, RecentType.Swap)
}

private fun SavedStateHandle.requireSwapItemType(): SwapItemType =
    checkNotNull(get<SwapItemType>(RouteArgument.SwapItemType.key)) {
        "Missing route argument: ${RouteArgument.SwapItemType.key}"
    }

@OptIn(ExperimentalCoroutinesApi::class)
class SwapSelectSearch(
    private val searchSwapAssets: SearchSwapAssets,
) : SelectSearch {

    val swapItemType = MutableStateFlow<SwapItemType?>(null)
    val payId = MutableStateFlow<AssetId?>(null)
    val receiveId = MutableStateFlow<AssetId?>(null)

    override fun items(filters: Flow<SelectAssetFilters?>): Flow<List<AssetInfo>> {
        return combine(filters, swapItemType, payId, receiveId) { filter, type, payId, receiveId ->
            SearchInputs(
                filter = filter,
                type = type,
                oppositeAssetId = getOppositeAssetId(type, payId, receiveId),
            )
        }
        .flatMapLatest { inputs ->
            searchSwapAssets(
                wallet = inputs.filter?.session?.wallet,
                query = inputs.filter?.query ?: "",
                swapItemType = inputs.type ?: SwapItemType.Receive,
                oppositeAssetId = inputs.oppositeAssetId,
                tag = inputs.filter?.tag,
            )
        }
    }

    private fun getOppositeAssetId(type: SwapItemType?, payId: AssetId?, receiveId: AssetId?) = when (type) {
        SwapItemType.Pay -> receiveId
        SwapItemType.Receive -> payId
        null -> null
    }

    private data class SearchInputs(
        val filter: SelectAssetFilters?,
        val type: SwapItemType?,
        val oppositeAssetId: AssetId?,
    )
}
