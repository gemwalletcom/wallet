package com.gemwallet.android.features.price_alerts.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.price_alerts.viewmodels.AssetPriceAlertsViewModel
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.gemwallet.android.ui.models.navigation.RouteMessage
import com.wallet.core.primitives.AssetId

@Composable
fun AssetPriceAlertsScreen(message: RouteMessage?, onMessageShown: () -> Unit, onSetPriceAlert: (AssetId) -> Unit, onCancel: () -> Unit, viewModel: AssetPriceAlertsViewModel = hiltViewModel()) {
    val error by viewModel.error.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = message, onShown = onMessageShown)

    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val assetAlerts by viewModel.assetAlerts.collectAsStateWithLifecycle()
    val isRefreshing by viewModel.isRefreshing.collectAsStateWithLifecycle()

    PriceAlertsScene(
        sections = sections,
        phase = assetAlerts?.phase,
        syncState = isRefreshing,
        snackbar = snackbar,
        error = error,
        onErrorShown = viewModel::clearError,
        header = { assetAlerts?.let { autoAlertToggle(it, viewModel::toggleAutoAlert) } },
        onChart = null,
    ) { action ->
        when (action) {
            PriceAlertsAction.Refresh -> viewModel.refresh()
            PriceAlertsAction.Close -> onCancel()
            PriceAlertsAction.Add -> onSetPriceAlert(viewModel.assetId)
            is PriceAlertsAction.Exclude -> viewModel.excludeAsset(action.id)
        }
    }
}
