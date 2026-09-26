package com.gemwallet.android.ui.components.screen

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.SearchBar
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_item.ChainItem
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemEmptyStateKind
import uniffi.gemstone.chainRow

@Composable
fun SelectChain(
    chains: List<Chain>,
    chainFilter: TextFieldState,
    listState: LazyListState = rememberLazyListState(),
    title: String = stringResource(id = R.string.settings_networks_title),
    trailing: (@Composable (Chain) -> Unit)? = null,
    listHeader: LazyListScope.() -> Unit = {},
    onSelect: (Chain) -> Unit,
    onCancel: () -> Unit,
) {
    Scene(
        title = title,
        onClose = onCancel,
    ) {
        LazyColumn(modifier = Modifier, state = listState) {
            item {
                SearchBar(query = chainFilter)
            }
            listHeader()
            if (chains.isEmpty()) {
                item {
                    EmptyContentView(
                        kind = GemEmptyStateKind.SEARCH_NETWORKS,
                        modifier = Modifier.fillParentMaxSize(),
                    )
                }
            } else {
                val size = chains.size
                itemsIndexed(chains) { index, item ->
                    ChainItem(
                        row = chainRow(item.string),
                        listPosition = ListPosition.getPosition(index, size),
                        trailing = trailing?.let { t -> @Composable { t(item) } },
                        onClick = { onSelect(item) },
                    )
                }
            }
        }
    }
}
