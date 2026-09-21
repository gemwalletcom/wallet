package com.gemwallet.android.features.settings.currency.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.currency.viewmodels.CurrenciesViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.SearchBar
import com.gemwallet.android.ui.components.empty.EmptyStateView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.listSections
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.rememberSnackbarState

@Composable
fun CurrenciesScene(onCancel: () -> Unit, viewModel: CurrenciesViewModel = hiltViewModel()) {
    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val error by viewModel.error.collectAsStateWithLifecycle()
    val listState = rememberLazyListState()
    val snackbar = rememberSnackbarState(message = error, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)

    LaunchedEffect(sections) {
        listState.scrollToItem(0)
    }

    Scene(
        title = stringResource(id = R.string.settings_currency),
        snackbar = snackbar,
        onClose = onCancel,
    ) {
        SearchBar(query = viewModel.query)
        if (sections.isEmpty() && viewModel.query.text.isNotBlank()) {
            EmptyStateView(title = stringResource(R.string.common_no_results_found), modifier = Modifier.fillMaxSize())
        }
        LazyColumn(state = listState) {
            listSections(sections, key = { it.currency.string }) { position, row ->
                ListItem(
                    model = row.model,
                    listPosition = position,
                    modifier = Modifier.clickable {
                        viewModel.setCurrency(row.currency, onCancel)
                    },
                    minHeight = ListItemDefaults.plainMinHeight,
                    accessory = if (row.isSelected) {
                        { SelectionCheckmark() }
                    } else {
                        null
                    },
                )
            }
        }
    }
}
