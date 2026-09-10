package com.gemwallet.android.features.perpetual.views.position

import com.gemwallet.android.ui.components.screen.SheetExpansion
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.LifecycleResumeEffect
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.perpetual.viewmodels.PerpetualDetailsViewModel
import com.gemwallet.android.features.perpetual.views.autoclose.AutocloseNavGraph
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.wallet.core.primitives.TransactionId
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.gemwallet.android.features.confirm.presents.AcquireAssetAction
import com.wallet.core.primitives.AssetId

@Composable
fun PerpetualPositionNavScreen(
    amountAction: AmountTransactionAction,
    confirmAction: ConfirmTransactionAction,
    onClose: () -> Unit,
    onTransaction: (TransactionId) -> Unit,
    onAcquireAsset: (AcquireAssetAction, AssetId) -> Unit,
    viewModel: PerpetualDetailsViewModel = hiltViewModel(),
) {
    LifecycleResumeEffect(Unit) {
        viewModel.fetch()
        onPauseOrDispose { }
    }

    DisposableEffect(Unit) {
        viewModel.onScreenEnter()
        onDispose { viewModel.onScreenExit() }
    }

    val perpetual by viewModel.perpetual.collectAsStateWithLifecycle()
    val position by viewModel.position.collectAsStateWithLifecycle()
    val transactions by viewModel.transactions.collectAsStateWithLifecycle()
    val chart by viewModel.chart.collectAsStateWithLifecycle()
    val period by viewModel.period.collectAsStateWithLifecycle()
    val isRefreshing by viewModel.isRefreshing.collectAsStateWithLifecycle()
    val error by viewModel.error.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = error, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)
    var showAutoclose by remember { mutableStateOf(false) }

    PerpetualPositionScene(
        perpetual = perpetual,
        position = position,
        transactions = transactions,
        chart = chart,
        period = period,
        isRefreshing = isRefreshing,
        snackbar = snackbar,
        onAction = { action ->
            when (action) {
                PerpetualDetailsAction.Close -> onClose()
                PerpetualDetailsAction.Refresh -> viewModel.refresh()
                PerpetualDetailsAction.IncreasePosition -> viewModel.increasePosition(amountAction)
                PerpetualDetailsAction.ReducePosition -> viewModel.reducePosition(amountAction)
                PerpetualDetailsAction.ClosePosition -> viewModel.closePosition(confirmAction)
                PerpetualDetailsAction.Autoclose -> showAutoclose = true
                is PerpetualDetailsAction.OpenPosition -> viewModel.openPosition(action.direction, amountAction)
                is PerpetualDetailsAction.SelectChartPeriod -> viewModel.period(action.period)
                is PerpetualDetailsAction.OpenTransaction -> onTransaction(action.transactionId)
            }
        },
    )

    ModalBottomSheet(
        isVisible = showAutoclose,
        onDismissRequest = { showAutoclose = false },
        expansion = SheetExpansion.Full,
        title = null,
        dragHandle = null,
    ) {
        AutocloseNavGraph(
            onDismiss = { showAutoclose = false },
            finishAction = FinishConfirmAction { _ -> viewModel.fetch() },
            onAcquireAsset = onAcquireAsset,
        )
    }
}
