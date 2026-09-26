package com.gemwallet.android.features.assets.presents.asset

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.assets.viewmodels.asset.AssetViewModel
import com.gemwallet.android.features.assets.viewmodels.asset.models.AssetAction
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.RefreshOnTimer
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.ToastEffect
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.gemwallet.android.ui.open

@Composable
fun AssetScreen(onAction: (AssetAction.Navigation) -> Unit) {
    val viewModel: AssetViewModel = hiltViewModel()
    val isRefreshing by viewModel.isRefreshing.collectAsStateWithLifecycle()
    val transactions by viewModel.transactions.collectAsStateWithLifecycle()
    val transactionsErrorRow by viewModel.transactionsErrorRow.collectAsStateWithLifecycle()
    val priceAlertError by viewModel.error.collectAsStateWithLifecycle()
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current
    val snackBar = rememberSnackbarState(message = priceAlertError, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)
    ToastEffect(viewModel.toastEvents, snackBar)
    val details by viewModel.details.collectAsStateWithLifecycle()
    val asset by viewModel.asset.collectAsStateWithLifecycle()

    val refreshIntervalMillis by viewModel.refreshIntervalMillis.collectAsStateWithLifecycle()
    RefreshOnTimer(refreshIntervalMillis, viewModel::refresh)

    val currentDetails = details
    val currentAsset = asset
    if (currentDetails != null && currentAsset != null) {
        AssetScene(
            details = currentDetails,
            asset = currentAsset,
            transactions = transactions,
            transactionsErrorRow = transactionsErrorRow,
            isRefreshing = isRefreshing,
            snackBar = snackBar,
            onAction = { action ->
                when (action) {
                    AssetAction.Refresh -> viewModel.refresh()

                    AssetAction.Pin -> viewModel.pin()

                    AssetAction.Add -> viewModel.add()

                    is AssetAction.TogglePriceAlert -> viewModel.togglePriceAlert(action.assetId)

                    is AssetAction.CloseBanner -> viewModel.closeBanner(action.key)

                    is AssetAction.OpenUrl -> uriHandler.open(context, action.url)

                    AssetAction.OpenPerpetuals -> {
                        viewModel.enablePerpetuals()
                        onAction(AssetAction.OpenPerpetuals)
                    }

                    is AssetAction.Navigation -> onAction(action)
                }
            },
        )
    } else {
        LoadingScene(
            title = stringResource(R.string.common_loading),
            onCancel = { onAction(AssetAction.Close) },
        )
    }
}
