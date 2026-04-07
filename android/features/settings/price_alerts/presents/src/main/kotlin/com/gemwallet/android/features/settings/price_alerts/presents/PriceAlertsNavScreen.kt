package com.gemwallet.android.features.settings.price_alerts.presents

import androidx.compose.animation.AnimatedContent
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.pricealerts.values.PriceAlertsStateEvent
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.gemwallet.android.features.settings.price_alerts.viewmodels.PriceAlertViewModel
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.launch

@Composable
fun PriceAlertsNavScreen(
    toastMessage: String? = null,
    onToastShown: () -> Unit = {},
    onChart: (AssetId) -> Unit,
    onAddPriceAlertTarget: (AssetId) -> Unit,
    onCancel: () -> Unit,
    viewModel: PriceAlertViewModel = hiltViewModel(),
) {
    val resources = LocalContext.current.resources
    val scope = rememberCoroutineScope()
    val snackbar = rememberSnackbarState(message = toastMessage, onShown = onToastShown)

    var selectingAsset by remember { mutableStateOf(false) }

    val data by viewModel.data.collectAsStateWithLifecycle()
    val priceAlertState by viewModel.priceAlertState.collectAsStateWithLifecycle()
    val isRefreshing by viewModel.isRefreshing.collectAsStateWithLifecycle()
    val assetId by viewModel.assetId.collectAsStateWithLifecycle()

    AnimatedContent(selectingAsset, label = "") { selecting ->
        when (selecting) {
            true -> PriceAlertSelectScreen(
                onCancel = { selectingAsset = false },
                onSelect = {
                    viewModel.includeAsset(it) { asset ->
                        val message = resources.getString(R.string.price_alerts_enabled_for, asset.name)
                        scope.launch {
                            snackbar.showSnackbar(message)
                        }
                    }
                    selectingAsset = false
                },
            )
            false -> PriceAlertScene(
                assetId = assetId,
                data = data,
                enabled = priceAlertState is PriceAlertsStateEvent.Enable,
                syncState = isRefreshing,
                isAssetView = viewModel.isAssetManage(),
                snackbar = snackbar,
                onEnablePriceAlerts = viewModel::togglePriceAlerts,
                onChart = onChart,
                onExclude = viewModel::excludeAsset,
                onRefresh = viewModel::refresh,
                onCancel = onCancel,
                onAdd = { selectingAsset = true },
                onAddTarget = onAddPriceAlertTarget,
            )
        }
    }

}
