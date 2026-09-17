package com.gemwallet.android.ui.components.filters

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.SearchBar
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.filters.model.FilterType
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemSymbol
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.gemwallet.android.ui.localization.getLabel
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemTransactionFilter
import uniffi.gemstone.transactionFilters

@Composable
fun TransactionsFilter(
    isVisible: Boolean,
    availableChains: List<Chain>,
    chainsFilter: List<Chain>,
    typesFilter: List<GemTransactionFilter>,
    onDismissRequest: () -> Unit,
    onSelectChainsFilter: (List<Chain>) -> Unit,
    onSelectTypesFilter: (List<GemTransactionFilter>) -> Unit,
    onClearChainsFilter: () -> Unit,
    onClearTypesFilter: () -> Unit,
) {
    var showedSubFilter by remember { mutableStateOf<FilterType?>(null) }

    FormDialog(
        isVisible = isVisible,
        title = stringResource(R.string.filter_title),
        onDismiss = onDismissRequest,
        onClear = {
            onClearChainsFilter()
            onClearTypesFilter()
        }.takeIf { typesFilter.isNotEmpty() || chainsFilter.isNotEmpty() },
    ) {
        LazyColumn {
            item {
                ListItem(
                    model = ListItemModel(
                        title = stringResource(R.string.settings_networks_title),
                        subtitle = when {
                            chainsFilter.isEmpty() -> stringResource(R.string.common_all)
                            chainsFilter.size == 1 -> chainsFilter.firstOrNull()?.networkName() ?: ""
                            else -> "${chainsFilter.size}"
                        },
                        image = ListItemImage.Drawable(R.drawable.settings_networks),
                    ),
                    listPosition = ListPosition.First,
                    modifier = Modifier.clickable { showedSubFilter = FilterType.ByChains },
                    accessory = { DataBadgeChevron() },
                )
            }
            item {
                ListItem(
                    model = ListItemModel(
                        title = stringResource(R.string.filter_types),
                        subtitle = when {
                            typesFilter.isEmpty() -> stringResource(R.string.common_all)
                            typesFilter.size == 1 -> typesFilter.firstOrNull()?.getLabel()?.let { stringResource(it) } ?: ""
                            else -> "${typesFilter.size}"
                        },
                        image = ListItemImage.Symbol(ListItemSymbol.Article),
                    ),
                    listPosition = ListPosition.Last,
                    modifier = Modifier.clickable { showedSubFilter = FilterType.ByTypes },
                    accessory = { DataBadgeChevron() },
                )
            }
        }
    }

    SubFilterDialog(
        isVisible = showedSubFilter == FilterType.ByChains,
        initialSelection = chainsFilter,
        onDone = {
            onSelectChainsFilter(it)
            showedSubFilter = null
        },
        onConfirm = {
            onSelectChainsFilter(it)
            showedSubFilter = null
            onDismissRequest()
        },
        onDismiss = { showedSubFilter = null },
    ) { selectedItems, onToggle ->
        val query = rememberTextFieldState()
        val matchingChains = rememberMatchingChains(availableChains, query.text.toString())
        SearchBar(query)
        LazyColumn(modifier = Modifier.fillMaxSize()) {
            selectFilterChain(matchingChains, selectedItems, onToggle)
        }
    }
    SubFilterDialog(
        isVisible = showedSubFilter == FilterType.ByTypes,
        initialSelection = typesFilter,
        onDone = {
            onSelectTypesFilter(it)
            showedSubFilter = null
        },
        onConfirm = {
            onSelectTypesFilter(it)
            showedSubFilter = null
            onDismissRequest()
        },
        onDismiss = { showedSubFilter = null },
    ) { selectedItems, onToggle ->
        val filters = remember { transactionFilters() }
        LazyColumn(modifier = Modifier.fillMaxSize()) {
            selectFilterTransactionType(filters, selectedItems, onToggle)
        }
    }
}

@Composable
private fun <T> SubFilterDialog(
    isVisible: Boolean,
    initialSelection: List<T>,
    onDone: (List<T>) -> Unit,
    onConfirm: (List<T>) -> Unit,
    onDismiss: () -> Unit,
    content: @Composable ColumnScope.(selectedItems: List<T>, onToggle: (T) -> Unit) -> Unit,
) {
    var selectedItems by remember { mutableStateOf(initialSelection) }
    FormDialog(
        isVisible = isVisible,
        title = stringResource(R.string.filter_title),
        expansion = SheetExpansion.Full,
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
