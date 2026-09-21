package com.gemwallet.android.features.asset.presents.chart

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.pulltorefresh.PullToRefreshDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.asset.viewmodels.chart.viewmodels.AssetChartViewModel
import com.gemwallet.android.features.asset.viewmodels.chart.viewmodels.ChartViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.RefreshOnTimer
import com.gemwallet.android.ui.components.list_item.gemListSections
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemListRowTitle

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AssetChartScene(
    onCancel: () -> Unit,
    onPriceAlerts: (AssetId) -> Unit,
    onAddPriceAlertTarget: (AssetId) -> Unit,
    toastMessage: String? = null,
    onToastShown: () -> Unit = {},
    viewModel: AssetChartViewModel = hiltViewModel(),
    chartViewModel: ChartViewModel = hiltViewModel(),
) {
    val refreshIntervalMillis by chartViewModel.refreshIntervalMillis.collectAsStateWithLifecycle()
    RefreshOnTimer(refreshIntervalMillis, chartViewModel::refresh)

    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val title by viewModel.title.collectAsStateWithLifecycle()
    val isChartRefreshing by chartViewModel.isRefreshing.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(
        message = toastMessage,
        iconRes = R.drawable.ic_notifications,
        onShown = onToastShown,
    )

    Scene(
        title = title,
        onClose = onCancel,
        snackbar = snackbar,
    ) {
        PullToRefreshBox(
            isRefreshing = isChartRefreshing,
            onRefresh = {
                chartViewModel.refresh()
            },
            containerColor = PullToRefreshDefaults.indicatorContainerColor,
        ) {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                item { Chart(chartViewModel) }
                gemListSections(sections) { title ->
                    when (title) {
                        GemListRowTitle.PRICE_ALERTS -> onPriceAlerts(viewModel.assetId)
                        GemListRowTitle.SET_PRICE_ALERT -> onAddPriceAlertTarget(viewModel.assetId)
                        else -> Unit
                    }
                }
            }
        }
    }
}
