package com.gemwallet.android.features.price_alerts.presents

import androidx.compose.animation.AnimatedContent
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.price_alerts.viewmodels.PriceAlertsViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.message
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.gemwallet.android.ui.components.screen.showSnackbar
import com.gemwallet.android.ui.models.navigation.RouteMessage
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.launch

@Composable
fun PriceAlertsScreen(message: RouteMessage?, onMessageShown: () -> Unit, onChart: (AssetId) -> Unit, onSetPriceAlert: (AssetId) -> Unit, onCancel: () -> Unit, viewModel: PriceAlertsViewModel = hiltViewModel()) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val snackbar = rememberSnackbarState(message = message, onShown = onMessageShown)
    val error by viewModel.error.collectAsStateWithLifecycle()
    val errorMessage = error
    LaunchedEffect(errorMessage) {
        errorMessage?.let {
            snackbar.showSnackbar(it, R.drawable.ic_error)
            viewModel.clearError()
        }
    }

    var selectingAsset by remember { mutableStateOf(false) }

    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val assetAlerts by viewModel.assetAlerts.collectAsStateWithLifecycle()
    val showsEmpty by viewModel.showsEmpty.collectAsStateWithLifecycle()
    val asset by viewModel.asset.collectAsStateWithLifecycle()
    val priceAlertEnabled by viewModel.priceAlertEnabled.collectAsStateWithLifecycle()
    val isRefreshing by viewModel.isRefreshing.collectAsStateWithLifecycle()
    val errorRow by viewModel.errorRow.collectAsStateWithLifecycle()

    AnimatedContent(selectingAsset, label = "") { selecting ->
        when (selecting) {
            true -> AddAssetPriceAlertsScreen(
                onCancel = { selectingAsset = false },
                onSelect = { assetId ->
                    viewModel.includeAsset(assetId) { toast ->
                        val message = toast.message(context)
                        scope.launch {
                            snackbar.showSnackbar(message.title, message.image)
                        }
                    }
                    selectingAsset = false
                },
            )

            false -> PriceAlertsScene(
                errorRow = errorRow,
                asset = asset,
                sections = sections,
                assetAlerts = assetAlerts,
                showsEmpty = showsEmpty,
                enabled = priceAlertEnabled == true,
                syncState = isRefreshing,
                isAssetView = viewModel.isAssetManage(),
                snackbar = snackbar,
                onAction = { action ->
                    when (action) {
                        is PriceAlertsAction.TogglePriceAlerts -> viewModel.togglePriceAlerts(action.enabled)
                        is PriceAlertsAction.ToggleAutoAlert -> viewModel.toggleAutoAlert(action.enabled)
                        is PriceAlertsAction.Exclude -> viewModel.excludeAsset(action.id)
                        PriceAlertsAction.Refresh -> viewModel.refresh()
                        PriceAlertsAction.Add -> selectingAsset = true
                        PriceAlertsAction.Close -> onCancel()
                        is PriceAlertsAction.OpenChart -> onChart(action.assetId)
                        is PriceAlertsAction.SetPriceAlert -> onSetPriceAlert(action.assetId)
                    }
                },
            )
        }
    }
}
