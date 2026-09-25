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
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.activities.viewmodels.TransactionsFilterSummaryUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.filters.TransactionFilterUIModel
import com.gemwallet.android.ui.components.filters.TransactionsFilter
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.rememberDateSections
import com.gemwallet.android.ui.components.list_item.transaction.transactionsList
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.space0
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemEmptyStateAction
import uniffi.gemstone.GemEmptyStateKind
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemTransactionRow

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun TransactionsScene(
    isRefreshing: Boolean,
    transactions: List<GemTransactionRow>?,
    errorRow: GemListRow?,
    availableChains: List<Chain>,
    chainsFilter: List<Chain>,
    typeFilter: List<TransactionFilterUIModel>,
    typeFilterOptions: List<TransactionFilterUIModel>,
    filterSummary: TransactionsFilterSummaryUIModel,
    emptyStateKind: GemEmptyStateKind,
    listState: LazyListState = rememberLazyListState(),
    showBuyAction: Boolean,
    showReceiveAction: Boolean,
    onAction: (TransactionsListAction) -> Unit,
) {
    var showFilters by remember { mutableStateOf(false) }

    Scene(
        title = stringResource(id = R.string.activity_title),
        mainActionPadding = PaddingValues(space0),
        actions = {
            IconButton(onClick = { showFilters = !showFilters }) {
                Icon(
                    imageVector = AppIcons.FilterAlt,
                    tint = if (chainsFilter.isEmpty() && typeFilter.isEmpty()) {
                        LocalContentColor.current
                    } else {
                        MaterialTheme.colorScheme.primary
                    },
                    contentDescription = "Filter by networks",
                )
            }
        },
    ) {
        val transactionSections = rememberDateSections(transactions.orEmpty()) { it.createdAt }
        PullToRefreshBox(
            isRefreshing = isRefreshing,
            onRefresh = { onAction(TransactionsListAction.Refresh) },
        ) {
            when {
                transactions == null -> Unit

                errorRow != null -> LazyColumn(modifier = Modifier.fillMaxSize()) {
                    item { GemListRowView(row = errorRow, listPosition = ListPosition.Single) }
                }

                transactions.isEmpty() -> LazyColumn(modifier = Modifier.fillMaxSize()) {
                    item {
                        EmptyContentView(
                            type = transactionsEmptyContentType(
                                kind = emptyStateKind,
                                showBuyAction = showBuyAction,
                                showReceiveAction = showReceiveAction,
                                onAction = onAction,
                            ),
                            modifier = Modifier.fillParentMaxSize(),
                        )
                    }
                }

                else -> LazyColumn(
                    modifier = Modifier.fillMaxSize(),
                    state = listState,
                ) {
                    transactionsList(
                        sections = transactionSections,
                        onTransactionClick = { onAction(TransactionsListAction.OpenTransaction(it)) },
                    )
                }
            }
        }
    }
    TransactionsFilter(
        isVisible = showFilters,
        availableChains = availableChains,
        chainsFilter = chainsFilter,
        typesFilter = typeFilter,
        typeOptions = typeFilterOptions,
        chainsSummary = filterSummary.chains,
        typesSummary = filterSummary.types,
        onDismissRequest = { showFilters = false },
        onSelectChainsFilter = { onAction(TransactionsListAction.SelectChainsFilter(it)) },
        onSelectTypesFilter = { onAction(TransactionsListAction.SelectTypesFilter(it)) },
        onClearChainsFilter = { onAction(TransactionsListAction.ClearChainsFilter) },
        onClearTypesFilter = { onAction(TransactionsListAction.ClearTypesFilter) },
    )
}

private fun transactionsEmptyContentType(kind: GemEmptyStateKind, showBuyAction: Boolean, showReceiveAction: Boolean, onAction: (TransactionsListAction) -> Unit): EmptyContentType {
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

    val onClearFilters = {
        onAction(TransactionsListAction.ClearChainsFilter)
        onAction(TransactionsListAction.ClearTypesFilter)
    }
    return EmptyContentType(kind, actions = mapOf(GemEmptyStateAction.BUY to onBuy, GemEmptyStateAction.RECEIVE to onReceive, GemEmptyStateAction.CLEAR_FILTERS to onClearFilters))
}
