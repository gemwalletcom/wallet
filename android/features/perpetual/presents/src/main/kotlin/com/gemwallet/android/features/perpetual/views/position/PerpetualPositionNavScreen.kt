package com.gemwallet.android.features.perpetual.views.position

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.perpetual.viewmodels.PerpetualDetailsViewModel
import com.wallet.core.primitives.TransactionId

@Composable
fun PerpetualPositionNavScreen(
    onClose: () -> Unit,
    onTransaction: (TransactionId) -> Unit,
    viewModel: PerpetualDetailsViewModel = hiltViewModel(),
) {
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
        onClose = onClose,
        onChartPeriodSelect = viewModel::period,
        onOpenPosition = {},
        onTransaction = onTransaction,
    )
}
