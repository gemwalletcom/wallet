package com.gemwallet.android.ui.components.filters

import androidx.compose.foundation.Image
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.Article
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.SearchBar
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.filters.model.FilterType
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.TransactionTypeFilter
import com.gemwallet.android.ui.theme.iconSize
import com.wallet.core.primitives.Chain

@Composable
fun TransactionsFilter(
    availableChains: List<Chain>,
    chainsFilter: List<Chain>,
    typesFilter: List<TransactionTypeFilter>,
    onDismissRequest: () -> Unit,
    onApplyChainsFilter: (List<Chain>) -> Unit,
    onApplyTypesFilter: (List<TransactionTypeFilter>) -> Unit,
    onClearChainsFilter: () -> Unit,
    onClearTypesFilter: () -> Unit,
) {
    var showedSubFilter by remember { mutableStateOf<FilterType?>(null) }

    FormDialog(
        title = stringResource(R.string.filter_title),
        onDismiss = onDismissRequest,
        onClear = {
            onClearChainsFilter()
            onClearTypesFilter()
        }.takeIf { typesFilter.isNotEmpty() || chainsFilter.isNotEmpty() },
    ) {
        LazyColumn {
            item {
                PropertyItem(
                    modifier = Modifier.clickable { showedSubFilter = FilterType.ByChains },
                    title = {
                        PropertyTitleText(
                            text = R.string.settings_networks_title,
                            trailing = {
                                Image(
                                    modifier = Modifier.size(iconSize),
                                    painter = painterResource(R.drawable.settings_networks),
                                    contentDescription = null,
                                )
                            }
                        )
                    },
                    data = {
                        PropertyDataText(
                            text = when {
                                chainsFilter.isEmpty() -> stringResource(R.string.common_all)
                                chainsFilter.size == 1 -> chainsFilter.firstOrNull()?.asset()?.name ?: ""
                                else -> "${chainsFilter.size}"
                            },
                            badge = { IconWithBadge(null) }
                        )
                    },
                    listPosition = ListPosition.First,
                )
            }
            item {
                PropertyItem(
                    modifier = Modifier.clickable { showedSubFilter = FilterType.ByTypes },
                    title = {
                        PropertyTitleText(
                            text = R.string.filter_types,
                            trailing = {
                                Icon(
                                    modifier = Modifier.size(iconSize),
                                    imageVector = Icons.AutoMirrored.Default.Article,
                                    contentDescription = null,
                                )
                            }
                        )
                    },
                    data = {
                        PropertyDataText(
                            text = when {
                                typesFilter.isEmpty() -> stringResource(R.string.common_all)
                                typesFilter.size == 1 -> typesFilter.firstOrNull()?.getLabel()
                                    ?.let { stringResource(it) } ?: ""

                                else -> "${typesFilter.size}"
                            },
                            badge = { IconWithBadge(null) }
                        )
                    },
                    listPosition = ListPosition.Last,
                )
            }
        }
    }

    when (showedSubFilter) {
        FilterType.ByChains -> SubFilterDialog(
            initialSelection = chainsFilter,
            onDone = {
                onApplyChainsFilter(it)
                showedSubFilter = null
            },
            onConfirm = {
                onApplyChainsFilter(it)
                showedSubFilter = null
                onDismissRequest()
            },
            onDismiss = { showedSubFilter = null },
        ) { selectedItems, onToggle ->
            val query = rememberTextFieldState()
            SearchBar(query)
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                selectFilterChain(availableChains, selectedItems, query.text.toString(), onToggle)
            }
        }
        FilterType.ByTypes -> SubFilterDialog(
            initialSelection = typesFilter,
            onDone = {
                onApplyTypesFilter(it)
                showedSubFilter = null
            },
            onConfirm = {
                onApplyTypesFilter(it)
                showedSubFilter = null
                onDismissRequest()
            },
            onDismiss = { showedSubFilter = null },
        ) { selectedItems, onToggle ->
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                selectFilterTransactionType(selectedItems, onToggle)
            }
        }
        null -> {}
    }
}

@Composable
private fun <T> SubFilterDialog(
    initialSelection: List<T>,
    onDone: (List<T>) -> Unit,
    onConfirm: (List<T>) -> Unit,
    onDismiss: () -> Unit,
    content: @Composable ColumnScope.(selectedItems: List<T>, onToggle: (T) -> Unit) -> Unit,
) {
    var selectedItems by remember { mutableStateOf(initialSelection) }
    FormDialog(
        title = stringResource(R.string.filter_title),
        fullScreen = true,
        onDismiss = onDismiss,
        onClear = { selectedItems = emptyList() }.takeIf { selectedItems.isNotEmpty() },
        doneAction = {
            TextButton(onClick = { onDone(selectedItems) }) {
                Text(stringResource(R.string.common_done))
            }
        },
        bottomAction = {
            MainActionButton(
                title = stringResource(R.string.transfer_confirm),
                onClick = { onConfirm(selectedItems) },
            )
        },
    ) {
        content(selectedItems) { item ->
            selectedItems = selectedItems.toMutableList().apply {
                if (!remove(item)) add(item)
            }
        }
    }
}
