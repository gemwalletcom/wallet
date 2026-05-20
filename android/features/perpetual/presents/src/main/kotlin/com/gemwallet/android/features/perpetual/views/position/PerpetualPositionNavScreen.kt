package com.gemwallet.android.features.perpetual.views.position

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.perpetual.viewmodels.PerpetualDetailsViewModel
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.wallet.core.primitives.TransactionId

@Composable
fun PerpetualPositionNavScreen(
    amountAction: AmountTransactionAction,
    confirmAction: ConfirmTransactionAction,
    onClose: () -> Unit,
    onTransaction: (TransactionId) -> Unit,
    viewModel: PerpetualDetailsViewModel = hiltViewModel(),
) {
    LaunchedEffect(Unit) { viewModel.fetch() }

    val perpetual by viewModel.perpetual.collectAsStateWithLifecycle()
    val position by viewModel.position.collectAsStateWithLifecycle()
    val transactions by viewModel.transactions.collectAsStateWithLifecycle()
    val chart by viewModel.chart.collectAsStateWithLifecycle()
    val period by viewModel.period.collectAsStateWithLifecycle()

    PerpetualPositionScene(
        perpetual = perpetual,
        position = position,
        transactions = transactions,
        chartData = chart,
        period = period,
        onAction = { action ->
            when (action) {
                PerpetualDetailsAction.Close -> onClose()
                PerpetualDetailsAction.IncreasePosition -> viewModel.increasePosition(amountAction)
                PerpetualDetailsAction.ReducePosition -> viewModel.reducePosition(amountAction)
                PerpetualDetailsAction.ClosePosition -> viewModel.closePosition(confirmAction)
                is PerpetualDetailsAction.OpenPosition -> viewModel.openPosition(action.direction, amountAction)
                is PerpetualDetailsAction.SelectChartPeriod -> viewModel.period(action.period)
                is PerpetualDetailsAction.OpenTransaction -> onTransaction(action.transactionId)
            }
        },
    )
}
