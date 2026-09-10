package com.gemwallet.android.features.asset.presents.details

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.rememberNotificationPermissionGate
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.features.asset.viewmodels.details.viewmodels.AssetDetailsViewModel
import com.gemwallet.android.features.asset.viewmodels.details.viewmodels.AssetPriceAlertsViewModel
import com.gemwallet.android.ui.components.RefreshOnTimer
import uniffi.gemstone.GemRefreshKind
import com.gemwallet.android.ui.components.screen.rememberSnackbarState

@Composable
fun AssetDetailsScreen(
    onAction: (AssetDetailsAction.Navigation) -> Unit,
) {
    val viewModel: AssetDetailsViewModel = hiltViewModel()
    val priceAlertsViewModel: AssetPriceAlertsViewModel = hiltViewModel()
    val isRefreshing by viewModel.isRefreshing.collectAsStateWithLifecycle()
    val transactions by viewModel.transactions.collectAsStateWithLifecycle()
    val priceAlertEnabled by priceAlertsViewModel.isEnabled.collectAsStateWithLifecycle()
    val priceAlertsCount by priceAlertsViewModel.alertsCount.collectAsStateWithLifecycle()
    val priceAlertError by priceAlertsViewModel.error.collectAsStateWithLifecycle()
    val snackBar = rememberSnackbarState(message = priceAlertError, iconRes = R.drawable.ic_error, onShown = priceAlertsViewModel::clearError)
    val uiModel by viewModel.uiModel.collectAsStateWithLifecycle()
    val requestNotificationPermission = rememberNotificationPermissionGate()

    RefreshOnTimer(GemRefreshKind.WALLET, viewModel::refresh)

    if (uiModel != null) {
        AssetDetailsScene(
            uiState = uiModel ?: return,
            transactions = transactions,
            priceAlertEnabled = priceAlertEnabled == true,
            priceAlertsCount = priceAlertsCount,
            isRefreshing = isRefreshing,
            snackBar = snackBar,
            requestNotificationPermission = requestNotificationPermission,
            onAction = { action ->
                when (action) {
                    AssetDetailsAction.Refresh -> viewModel.refresh()
                    AssetDetailsAction.Pin -> viewModel.pin()
                    AssetDetailsAction.Add -> viewModel.add()
                    is AssetDetailsAction.TogglePriceAlert -> priceAlertsViewModel.toggle(action.assetId)
                    is AssetDetailsAction.Navigation -> onAction(action)
                }
            },
        )
    } else {
        LoadingScene(
            title = stringResource(R.string.common_loading),
            onCancel = { onAction(AssetDetailsAction.Close) },
        )
    }
}
