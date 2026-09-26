package com.gemwallet.android.features.swap.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.AssetsQuery
import com.gemwallet.android.data.services.store.queries.RecentActivityQuery
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.features.assets.viewmodels.select.BaseAssetSelectViewModel
import com.gemwallet.android.features.assets.viewmodels.select.models.BaseSelectSearch
import com.gemwallet.android.ui.models.navigation.RouteArgument
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import uniffi.gemstone.GemAssetSelectionServiceInterface
import uniffi.gemstone.GemSelectAssetType
import javax.inject.Inject

@HiltViewModel
class SwapSelectViewModel @Inject constructor(
    getSession: GetSession,
    assetsQuery: AssetsQuery,
    recentActivityQuery: RecentActivityQuery,
    service: GemAssetSelectionServiceInterface,
    savedStateHandle: SavedStateHandle,
    @IoDispatcher ioDispatcher: CoroutineDispatcher,
    @ApplicationContext context: Context,
) : BaseAssetSelectViewModel(
    getSession = getSession,
    recentActivityQuery = recentActivityQuery,
    service = service,
    search = BaseSelectSearch(assetsQuery),
    selectType = when (savedStateHandle.requireSwapItemType()) {
        SwapItemType.Pay -> GemSelectAssetType.SwapPay
        SwapItemType.Receive -> GemSelectAssetType.SwapReceive(payAssetId = savedStateHandle.get<String?>(RouteArgument.FromAssetId.key))
    },
    ioDispatcher,
    context,
) {

    val select = MutableStateFlow(savedStateHandle.requireSwapItemType())
}

private fun SavedStateHandle.requireSwapItemType(): SwapItemType = checkNotNull(get<SwapItemType>(RouteArgument.SwapItemType.key)) {
    "Missing route argument: ${RouteArgument.SwapItemType.key}"
}
