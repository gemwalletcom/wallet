package com.gemwallet.android.features.price_alerts.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.price_alerts.viewmodels.PriceAlertsViewModel
import com.gemwallet.android.ui.components.screen.ToastEffect
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.gemwallet.android.ui.models.navigation.RouteMessage
import com.wallet.core.primitives.AssetId

@Composable
fun PriceAlertsScreen(message: RouteMessage?, onMessageShown: () -> Unit, selectedAsset: AssetId?, onSelectedAssetConsumed: () -> Unit, onAction: (PriceAlertsAction.Navigation) -> Unit, viewModel: PriceAlertsViewModel = hiltViewModel()) {
    val error by viewModel.error.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = message, onShown = onMessageShown)

    val groups by viewModel.groups.collectAsStateWithLifecycle()
    val enabled by viewModel.priceAlertEnabled.collectAsStateWithLifecycle()
    val isRefreshing by viewModel.isRefreshing.collectAsStateWithLifecycle()
    val phase by viewModel.phase.collectAsStateWithLifecycle()

    ToastEffect(viewModel.toastEvents, snackbar)

    LaunchedEffect(selectedAsset) {
        val assetId = selectedAsset ?: return@LaunchedEffect
        viewModel.includeAsset(assetId)
        onSelectedAssetConsumed()
    }

    PriceAlertsScene(
        groups = groups,
        phase = phase,
        syncState = isRefreshing,
        snackbar = snackbar,
        error = error,
        onErrorShown = viewModel::clearError,
        header = { priceAlertsToggle(enabled, viewModel::togglePriceAlerts) },
        showsChart = true,
    ) { action ->
        when (action) {
            PriceAlertsAction.Refresh -> viewModel.refresh()
            is PriceAlertsAction.Exclude -> viewModel.excludeAsset(action.id)
            is PriceAlertsAction.Navigation -> onAction(action)
        }
    }
}
