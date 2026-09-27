package com.gemwallet.android.features.transactions.presents.list

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
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.filters.TransactionsFilter
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.rememberDateSections
import com.gemwallet.android.ui.components.list_item.transaction.transactionsList
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.space0
import uniffi.gemstone.GemEmptyStateAction
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemTransactionRow
import uniffi.gemstone.GemTransactionsFilterSession
import uniffi.gemstone.GemTransactionsFilterView

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun TransactionsScene(
    isRefreshing: Boolean,
    transactions: List<GemTransactionRow>?,
    errorRow: GemListRow?,
    filter: GemTransactionsFilterSession,
    filterView: GemTransactionsFilterView,
    listState: LazyListState = rememberLazyListState(),
    onAction: (TransactionsAction) -> Unit,
) {
    var showFilters by remember { mutableStateOf(false) }

    Scene(
        title = stringResource(id = R.string.activity_title),
        mainActionPadding = PaddingValues(space0),
        actions = {
            IconButton(onClick = { showFilters = !showFilters }) {
                Icon(
                    imageVector = AppIcons.FilterAlt,
                    tint = if (filterView.isFiltered) {
                        MaterialTheme.colorScheme.primary
                    } else {
                        LocalContentColor.current
                    },
                    contentDescription = "Filter by networks",
                )
            }
        },
    ) {
        val transactionSections = rememberDateSections(transactions.orEmpty()) { it.createdAt }
        PullToRefreshBox(
            isRefreshing = isRefreshing,
            onRefresh = { onAction(TransactionsAction.Refresh) },
        ) {
            when {
                transactions == null -> Unit

                errorRow != null -> LazyColumn(modifier = Modifier.fillMaxSize()) {
                    item { GemListRowView(row = errorRow, listPosition = ListPosition.Single) }
                }

                transactions.isEmpty() -> LazyColumn(modifier = Modifier.fillMaxSize()) {
                    item {
                        EmptyContentView(
                            state = filterView.emptyState,
                            onAction = { action ->
                                when (action) {
                                    GemEmptyStateAction.BUY -> onAction(TransactionsAction.Buy)
                                    GemEmptyStateAction.RECEIVE -> onAction(TransactionsAction.Receive)
                                    GemEmptyStateAction.CLEAR_FILTERS -> onAction(TransactionsAction.ClearFilters)
                                    GemEmptyStateAction.SWAP, GemEmptyStateAction.ADD_CUSTOM_TOKEN, GemEmptyStateAction.MANAGE_TOKEN_LIST -> Unit
                                }
                            },
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
                        onTransactionClick = { onAction(TransactionsAction.OpenTransaction(it)) },
                    )
                }
            }
        }
    }
    TransactionsFilter(
        isVisible = showFilters,
        filter = filter,
        view = filterView,
        onDismissRequest = { showFilters = false },
        onSelectChainsFilter = { onAction(TransactionsAction.SelectChainsFilter(it)) },
        onSelectTypesFilter = { onAction(TransactionsAction.SelectTypesFilter(it)) },
        onClear = { onAction(TransactionsAction.ClearFilters) },
    )
}
