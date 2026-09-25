package com.gemwallet.android.features.fiat_connect.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.fiat_connect.viewmodels.FiatTransactionsViewModel
import com.gemwallet.android.ui.models.actions.CancelAction

@Composable
fun FiatTransactionsNavScreen(onClose: CancelAction, viewModel: FiatTransactionsViewModel = hiltViewModel()) {
    val transactions by viewModel.transactions.collectAsStateWithLifecycle()
    val isRefreshing by viewModel.isRefreshing.collectAsStateWithLifecycle()
    val errorRow by viewModel.errorRow.collectAsStateWithLifecycle()

    FiatTransactionsScene(
        transactions = transactions,
        errorRow = errorRow,
        isRefreshing = isRefreshing,
        onClose = { onClose() },
        onRefresh = viewModel::refresh,
    )
}
