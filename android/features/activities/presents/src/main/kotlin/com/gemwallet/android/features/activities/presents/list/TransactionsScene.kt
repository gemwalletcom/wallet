package com.gemwallet.android.features.activities.presents.list

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.material3.pulltorefresh.PullToRefreshDefaults
import androidx.compose.material3.pulltorefresh.rememberPullToRefreshState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.gemwallet.android.domains.transaction.aggregates.TransactionDataAggregate
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.filters.TransactionsFilter
import com.gemwallet.android.ui.components.list_item.transaction.transactionsList
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.TransactionTypeFilter
import com.wallet.core.primitives.Chain

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun TransactionsScene(
    isRefreshing: Boolean,
    transactions: List<TransactionDataAggregate>,
    chainsFilter: List<Chain>,
    typeFilter: List<TransactionTypeFilter>,
    listState: LazyListState = rememberLazyListState(),
    showBuyAction: Boolean,
    showReceiveAction: Boolean,
    onAction: (TransactionsListAction) -> Unit,
) {
    val pullToRefreshState = rememberPullToRefreshState()
    var showFilters by remember { mutableStateOf(false) }

    Scene(
        title = stringResource(id = R.string.activity_title),
        mainActionPadding = PaddingValues(0.dp),
        actions = {
            IconButton(onClick = { showFilters = !showFilters }) {
                Icon(
                    imageVector = AppIcons.FilterAlt,
                    tint = if (chainsFilter.isEmpty() && typeFilter.isEmpty())
                        LocalContentColor.current
                    else
                        MaterialTheme.colorScheme.primary,
                    contentDescription = "Filter by networks",
                )
            }
        },
        navigationBarPadding = false,
    ) {
        PullToRefreshBox(
            modifier = Modifier,
            isRefreshing = isRefreshing,
            onRefresh = { onAction(TransactionsListAction.Refresh) },
            state = pullToRefreshState,
            indicator = {
                PullToRefreshDefaults.Indicator(
                    modifier = Modifier.align(Alignment.TopCenter),
                    isRefreshing = isRefreshing,
                    state = pullToRefreshState,
                    containerColor = MaterialTheme.colorScheme.background,
                )
            },
        ) {
            if (transactions.isEmpty()) {
                LazyColumn(modifier = Modifier.fillMaxSize()) {
                    item {
                        EmptyContentView(
                            type = transactionsEmptyContentType(
                                hasFilters = chainsFilter.isNotEmpty() || typeFilter.isNotEmpty(),
                                showBuyAction = showBuyAction,
                                showReceiveAction = showReceiveAction,
                                onAction = onAction,
                            ),
                            modifier = Modifier.fillParentMaxSize(),
                        )
                    }
                }
            } else {
                LazyColumn(
                    modifier = Modifier.fillMaxSize(),
                    state = listState,
                ) {
                    transactionsList(
                        items = transactions,
                        onTransactionClick = { onAction(TransactionsListAction.OpenTransaction(it)) },
                    )
                }
            }
        }
    }
    if (showFilters) {
        TransactionsFilter(
            availableChains = Chain.entries,
            chainsFilter = chainsFilter,
            typesFilter = typeFilter,
            onDismissRequest = { showFilters = false },
            onApplyChainsFilter = { onAction(TransactionsListAction.ApplyChainsFilter(it)) },
            onApplyTypesFilter = { onAction(TransactionsListAction.ApplyTypesFilter(it)) },
            onClearChainsFilter = { onAction(TransactionsListAction.ClearChainsFilter) },
            onClearTypesFilter = { onAction(TransactionsListAction.ClearTypesFilter) },
        )
    }
}

private fun transactionsEmptyContentType(
    hasFilters: Boolean,
    showBuyAction: Boolean,
    showReceiveAction: Boolean,
    onAction: (TransactionsListAction) -> Unit,
): EmptyContentType {
    if (hasFilters) {
        return EmptyContentType.SearchActivity {
            onAction(TransactionsListAction.ClearChainsFilter)
            onAction(TransactionsListAction.ClearTypesFilter)
        }
    }

    val onBuy: (() -> Unit)? = if (showBuyAction) {
        { onAction(TransactionsListAction.Buy) }
    } else {
        null
    }
    val onReceive: (() -> Unit)? = if (showReceiveAction) {
        { onAction(TransactionsListAction.Receive) }
    } else {
        null
    }

    return EmptyContentType.Activity(
        onBuy = onBuy,
        onReceive = onReceive,
    )
}
