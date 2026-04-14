package com.gemwallet.android.features.asset.presents.details

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.pricealerts.values.PriceAlertsStateEvent
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.models.actions.AssetIdAction
import com.gemwallet.android.features.asset.viewmodels.details.viewmodels.AssetDetailsViewModel
import com.wallet.core.primitives.AssetId

@Composable
fun AssetDetailsScreen(
    onCancel: () -> Unit,
    onTransfer: AssetIdAction,
    onReceive: (AssetId) -> Unit,
    onBuy: (AssetId) -> Unit,
    onSwap: (AssetId, AssetId?) -> Unit,
    onTransaction: (txId: String) -> Unit,
    onChart: (AssetId) -> Unit,
    openNetwork: AssetIdAction,
    onStake: (AssetId) -> Unit,
    onConfirm: (ConfirmParams) -> Unit,
    onPriceAlerts: (AssetId) -> Unit,
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
            onRefresh = viewModel::refresh,
            onTransfer = onTransfer,
            onBuy = onBuy,
            onSwap = onSwap,
            onReceive = onReceive,
            onTransaction = onTransaction,
            onChart = onChart,
            openNetwork = openNetwork,
            onStake = onStake,
            toggkePriceAlert = viewModel::enablePriceAlert,
            onPriceAlerts = onPriceAlerts,
            onConfirm = onConfirm,
            onPin = viewModel::pin,
            onAdd = viewModel::add,
            onCancel = onCancel,
        )
    } else {
        LoadingScene(stringResource(R.string.common_loading), onCancel)
    }
}
