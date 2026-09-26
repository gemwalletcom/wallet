package com.gemwallet.android.features.transactions.presents.list

import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.transactions.viewmodels.TransactionsViewModel
import com.gemwallet.android.ui.components.RefreshOnTimer
import com.wallet.core.primitives.TransactionId

@Composable
fun TransactionsScreen(onTransaction: (TransactionId) -> Unit, onBuy: (() -> Unit)? = null, onReceive: (() -> Unit)? = null, listState: LazyListState = rememberLazyListState(), viewModel: TransactionsViewModel = hiltViewModel()) {
    val transactions by viewModel.transactions.collectAsStateWithLifecycle()
    val isRefreshing by viewModel.isRefreshing.collectAsStateWithLifecycle()
    val chainFilter by viewModel.chainsFilter.collectAsStateWithLifecycle()
    val typeFilter by viewModel.typeFilterRows.collectAsStateWithLifecycle()
    val filterSummary by viewModel.filterSummary.collectAsStateWithLifecycle()
    val emptyStateKind by viewModel.emptyStateKind.collectAsStateWithLifecycle()
    val walletId by viewModel.walletId.collectAsStateWithLifecycle()
    val availableChains by viewModel.availableChains.collectAsStateWithLifecycle()
    val errorRow by viewModel.errorRow.collectAsStateWithLifecycle()

    LaunchedEffect(walletId) {
        viewModel.syncIfNeeded()
    }

    val refreshIntervalMillis by viewModel.refreshIntervalMillis.collectAsStateWithLifecycle()
    RefreshOnTimer(refreshIntervalMillis, viewModel::refresh)

    TransactionsScene(
        isRefreshing = isRefreshing,
        transactions = transactions,
        errorRow = errorRow,
        availableChains = availableChains,
        chainsFilter = chainFilter,
        typeFilter = typeFilter,
        typeFilterOptions = viewModel.typeFilterOptions,
        filterSummary = filterSummary,
        emptyStateKind = emptyStateKind,
        listState = listState,
        showBuyAction = onBuy != null,
        showReceiveAction = onReceive != null,
        onAction = { action ->
            when (action) {
                TransactionsAction.Refresh -> viewModel.refresh()
                is TransactionsAction.OpenTransaction -> onTransaction(action.transactionId)
                is TransactionsAction.SelectChainsFilter -> viewModel.setChainsFilter(action.chains)
                is TransactionsAction.SelectTypesFilter -> viewModel.setTypesFilter(action.types.map { it.filter })
                TransactionsAction.ClearChainsFilter -> viewModel.clearChainsFilter()
                TransactionsAction.ClearTypesFilter -> viewModel.clearTypeFilter()
                TransactionsAction.Buy -> onBuy?.invoke()
                TransactionsAction.Receive -> onReceive?.invoke()
            }
        },
    )
}
