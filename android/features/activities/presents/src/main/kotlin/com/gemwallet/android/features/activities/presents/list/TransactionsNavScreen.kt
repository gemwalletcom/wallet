package com.gemwallet.android.features.activities.presents.list

import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.activities.viewmodels.TransactionsViewModel
import com.wallet.core.primitives.TransactionId

@Composable
fun TransactionsNavScreen(
    onTransaction: (TransactionId) -> Unit,
    onBuy: (() -> Unit)? = null,
    onReceive: (() -> Unit)? = null,
    listState: LazyListState = rememberLazyListState(),
    viewModel: TransactionsViewModel = hiltViewModel()
) {
    val transactions by viewModel.transactions.collectAsStateWithLifecycle()
    val isRefreshing by viewModel.isRefreshing.collectAsStateWithLifecycle()
    val chainFilter by viewModel.chainsFilter.collectAsStateWithLifecycle()
    val typeFilter by viewModel.typeFilter.collectAsStateWithLifecycle()
    val walletId by viewModel.walletId.collectAsStateWithLifecycle()

    LaunchedEffect(walletId) {
        viewModel.syncIfNeeded()
    }

    TransactionsScene(
        isRefreshing = isRefreshing,
        transactions = transactions,
        chainsFilter = chainFilter,
        typeFilter = typeFilter,
        listState = listState,
        showBuyAction = onBuy != null,
        showReceiveAction = onReceive != null,
        onAction = { action ->
            when (action) {
                TransactionsListAction.Refresh -> viewModel.refresh()
                is TransactionsListAction.OpenTransaction -> onTransaction(action.transactionId)
                is TransactionsListAction.ApplyChainsFilter -> viewModel.applyChainsFilter(action.chains)
                is TransactionsListAction.ApplyTypesFilter -> viewModel.applyTypesFilter(action.types)
                TransactionsListAction.ClearChainsFilter -> viewModel.clearChainsFilter()
                TransactionsListAction.ClearTypesFilter -> viewModel.clearTypeFilter()
                TransactionsListAction.Buy -> onBuy?.invoke()
                TransactionsListAction.Receive -> onReceive?.invoke()
            }
        },
    )
}
