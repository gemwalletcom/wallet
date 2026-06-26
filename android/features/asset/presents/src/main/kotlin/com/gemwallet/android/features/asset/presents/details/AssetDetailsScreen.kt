package com.gemwallet.android.features.asset.presents.details

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.pricealerts.values.PriceAlertsStateEvent
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.features.asset.viewmodels.details.viewmodels.AssetDetailsViewModel

@Composable
fun AssetDetailsScreen(
    onAction: (AssetDetailsAction.Navigation) -> Unit,
) {
    val viewModel: AssetDetailsViewModel = hiltViewModel()
    val isRefreshing by viewModel.isRefreshing.collectAsStateWithLifecycle()
    val transactions by viewModel.transactions.collectAsStateWithLifecycle()
    val priceAlertEnabled by viewModel.priceAlertEnabled.collectAsStateWithLifecycle()
    val priceAlertsCount by viewModel.priceAlertsCount.collectAsStateWithLifecycle()
    val uiModel by viewModel.uiModel.collectAsStateWithLifecycle()
    val isOperationEnabled by viewModel.isOperationEnabled.collectAsStateWithLifecycle()

    if (uiModel != null) {
        AssetDetailsScene(
            uiState = uiModel ?: return,
            transactions = transactions,
            priceAlertEnabled = priceAlertEnabled is PriceAlertsStateEvent.Enable,
            priceAlertsCount = priceAlertsCount,
            isRefreshing = isRefreshing,
            isOperationEnabled = isOperationEnabled,
            onAction = { action ->
                when (action) {
                    AssetDetailsAction.Refresh -> viewModel.refresh()
                    AssetDetailsAction.Pin -> viewModel.pin()
                    AssetDetailsAction.Add -> viewModel.add()
                    is AssetDetailsAction.TogglePriceAlert -> viewModel.enablePriceAlert(action.assetId)
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
