package com.gemwallet.android.features.settings.presents.currency

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.settings.viewmodels.currency.models.CurrencyRowUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.SearchBar
import com.gemwallet.android.ui.components.empty.EmptyStateView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.listSections
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListSection

@Composable
fun CurrencyScene(sections: List<ListSection<CurrencyRowUIModel>>, query: TextFieldState, snackbar: SnackbarHostState, onSelect: (CurrencyRowUIModel) -> Unit, onCancel: () -> Unit) {
    val listState = rememberLazyListState()

    LaunchedEffect(sections) {
        listState.scrollToItem(0)
    }

    Scene(
        title = stringResource(id = R.string.settings_currency),
        snackbar = snackbar,
        onClose = onCancel,
    ) {
        SearchBar(query = query)
        if (sections.isEmpty() && query.text.isNotBlank()) {
            EmptyStateView(title = stringResource(R.string.common_no_results_found), modifier = Modifier.fillMaxSize())
        }
        LazyColumn(state = listState) {
            listSections(sections, key = { it.row.currency.name }) { position, row ->
                ListItem(
                    model = row.model,
                    listPosition = position,
                    modifier = Modifier.clickable {
                        onSelect(row)
                    },
                    minHeight = ListItemDefaults.plainMinHeight,
                    accessory = if (row.row.isSelected) {
                        { SelectionCheckmark() }
                    } else {
                        null
                    },
                )
            }
        }
    }
}
