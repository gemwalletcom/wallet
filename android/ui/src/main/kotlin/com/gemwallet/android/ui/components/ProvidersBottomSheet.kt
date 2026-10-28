package com.gemwallet.android.ui.components

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ProviderLoadingRowView
import com.gemwallet.android.ui.components.list_item.ProviderRowView
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemProviderKind
import uniffi.gemstone.GemProviderLoadingRow
import uniffi.gemstone.GemProviderRow

@Composable
fun ProvidersBottomSheet(providers: List<GemProviderRow>?, onSelect: (GemProviderKind) -> Unit, onDismiss: () -> Unit, pendingProviders: List<GemProviderLoadingRow> = emptyList()) {
    ModalBottomSheet(
        item = providers,
        onDismissRequest = onDismiss,
        title = { stringResource(R.string.buy_providers_title) },
    ) { providers ->
        ProvidersList(providers = providers, pendingProviders = pendingProviders, onSelect = onSelect)
    }
}

@Composable
fun ProvidersList(providers: List<GemProviderRow>, pendingProviders: List<GemProviderLoadingRow>, onSelect: (GemProviderKind) -> Unit) {
    val rows = providers.size + pendingProviders.size
    LazyColumn {
        itemsIndexed(providers) { index, row ->
            ProviderRowView(
                row = row,
                listPosition = ListPosition.getPosition(index, rows),
                onClick = { onSelect(row.kind) },
            )
        }
        itemsIndexed(pendingProviders) { index, row ->
            ProviderLoadingRowView(row = row, listPosition = ListPosition.getPosition(providers.size + index, rows))
        }
    }
}
