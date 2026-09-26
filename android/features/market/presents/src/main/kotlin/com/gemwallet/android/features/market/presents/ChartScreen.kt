package com.gemwallet.android.features.market.presents

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.pulltorefresh.PullToRefreshDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.market.viewmodels.ChartValuesViewModel
import com.gemwallet.android.features.market.viewmodels.ChartViewModel
import com.gemwallet.android.ui.components.RefreshOnTimer
import com.gemwallet.android.ui.components.list_item.gemListSections
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.gemwallet.android.ui.models.navigation.RouteMessage
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChainAddress
import uniffi.gemstone.GemRowAction

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ChartScreen(
    onCancel: () -> Unit,
    onPriceAlerts: (AssetId) -> Unit,
    onSetPriceAlert: (AssetId) -> Unit,
    onOpenAddress: (ChainAddress) -> Unit,
    message: RouteMessage?,
    onMessageShown: () -> Unit,
    viewModel: ChartViewModel = hiltViewModel(),
    valuesViewModel: ChartValuesViewModel = hiltViewModel(),
) {
    val refreshIntervalMillis by valuesViewModel.refreshIntervalMillis.collectAsStateWithLifecycle()
    RefreshOnTimer(refreshIntervalMillis, valuesViewModel::refresh)

    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val title by viewModel.title.collectAsStateWithLifecycle()
    val isChartRefreshing by valuesViewModel.isRefreshing.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = message, onShown = onMessageShown)

    Scene(
        title = title,
        onClose = onCancel,
        snackbar = snackbar,
    ) {
        PullToRefreshBox(
            isRefreshing = isChartRefreshing,
            onRefresh = {
                valuesViewModel.refresh()
            },
            containerColor = PullToRefreshDefaults.indicatorContainerColor,
        ) {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                item { Chart(valuesViewModel) }
                gemListSections(sections, onSelectAddress = { onOpenAddress(ChainAddress(viewModel.assetId.chain, it)) }) { action ->
                    when (action) {
                        GemRowAction.PriceAlerts -> onPriceAlerts(viewModel.assetId)
                        GemRowAction.SetPriceAlert -> onSetPriceAlert(viewModel.assetId)
                        else -> Unit
                    }
                }
            }
        }
    }
}
