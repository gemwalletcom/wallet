package com.gemwallet.android.features.price_alerts.presents

import androidx.compose.animation.AnimatedContent
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.price_alerts.viewmodels.PriceAlertsViewModel
import com.gemwallet.android.ui.components.screen.message
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.gemwallet.android.ui.components.screen.showSnackbar
import com.gemwallet.android.ui.models.navigation.RouteMessage
import com.wallet.core.primitives.AssetId

@Composable
fun PriceAlertsScreen(message: RouteMessage?, onMessageShown: () -> Unit, onChart: (AssetId) -> Unit, onCancel: () -> Unit, viewModel: PriceAlertsViewModel = hiltViewModel()) {
    val context = LocalContext.current
    val error by viewModel.error.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = message, onShown = onMessageShown)

    var selectingAsset by rememberSaveable { mutableStateOf(false) }

    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val enabled by viewModel.priceAlertEnabled.collectAsStateWithLifecycle()
    val isRefreshing by viewModel.isRefreshing.collectAsStateWithLifecycle()
    val phase by viewModel.phase.collectAsStateWithLifecycle()
    val toast by viewModel.toast.collectAsStateWithLifecycle()

    LaunchedEffect(toast) {
        toast?.let {
            viewModel.toast.value = null
            val message = it.message(context)
            snackbar.showSnackbar(message.title, message.image)
        }
    }

    AnimatedContent(selectingAsset, label = "") { selecting ->
        when (selecting) {
            true -> AddAssetPriceAlertsScreen(
                onCancel = { selectingAsset = false },
                onSelect = { assetId ->
                    viewModel.includeAsset(assetId)
                    selectingAsset = false
                },
            )

            false -> PriceAlertsScene(
                sections = sections,
                phase = phase,
                syncState = isRefreshing,
                snackbar = snackbar,
                error = error,
                onErrorShown = viewModel::clearError,
                header = { priceAlertsToggle(enabled, viewModel::togglePriceAlerts) },
                onChart = onChart,
            ) { action ->
                when (action) {
                    PriceAlertsAction.Refresh -> viewModel.refresh()
                    PriceAlertsAction.Close -> onCancel()
                    PriceAlertsAction.Add -> selectingAsset = true
                    is PriceAlertsAction.Exclude -> viewModel.excludeAsset(action.id)
                }
            }
        }
    }
}
