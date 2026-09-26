package com.gemwallet.android.features.assets.presents.details

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.assets.viewmodels.details.models.AssetDetailsAction
import com.gemwallet.android.features.assets.viewmodels.details.viewmodels.AssetDetailsViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.RefreshOnTimer
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.ToastEffect
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.gemwallet.android.ui.open

@Composable
fun AssetDetailsScreen(onAction: (AssetDetailsAction.Navigation) -> Unit) {
    val viewModel: AssetDetailsViewModel = hiltViewModel()
    val isRefreshing by viewModel.isRefreshing.collectAsStateWithLifecycle()
    val transactions by viewModel.transactions.collectAsStateWithLifecycle()
    val transactionsErrorRow by viewModel.transactionsErrorRow.collectAsStateWithLifecycle()
    val priceAlertError by viewModel.error.collectAsStateWithLifecycle()
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current
    val snackBar = rememberSnackbarState(message = priceAlertError, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)
    ToastEffect(viewModel.toastEvents, snackBar)
    val uiModel by viewModel.uiModel.collectAsStateWithLifecycle()

    val refreshIntervalMillis by viewModel.refreshIntervalMillis.collectAsStateWithLifecycle()
    RefreshOnTimer(refreshIntervalMillis, viewModel::refresh)

    if (uiModel != null) {
        AssetDetailsScene(
            uiState = uiModel ?: return,
            transactions = transactions,
            transactionsErrorRow = transactionsErrorRow,
            isRefreshing = isRefreshing,
            snackBar = snackBar,
            onAction = { action ->
                when (action) {
                    AssetDetailsAction.Refresh -> viewModel.refresh()

                    AssetDetailsAction.Pin -> viewModel.pin()

                    AssetDetailsAction.Add -> viewModel.add()

                    is AssetDetailsAction.TogglePriceAlert -> viewModel.togglePriceAlert(action.assetId)

                    is AssetDetailsAction.CloseBanner -> viewModel.closeBanner(action.key)

                    is AssetDetailsAction.OpenUrl -> uriHandler.open(context, action.url)

                    AssetDetailsAction.OpenPerpetuals -> {
                        viewModel.enablePerpetuals()
                        onAction(AssetDetailsAction.OpenPerpetuals)
                    }

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
