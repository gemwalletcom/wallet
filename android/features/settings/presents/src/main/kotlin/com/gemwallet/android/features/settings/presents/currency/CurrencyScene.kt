package com.gemwallet.android.features.settings.presents.currency

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.SearchBar
import com.gemwallet.android.ui.components.list_item.gemGroupList
import com.gemwallet.android.ui.components.screen.Scene
import uniffi.gemstone.GemCurrencyRow
import uniffi.gemstone.GemGroupList
import uniffi.gemstone.GemListItem

@Composable
fun CurrencyScene(list: GemGroupList?, query: TextFieldState, snackbar: SnackbarHostState, onSelect: (GemCurrencyRow) -> Unit, onCancel: () -> Unit) {
    val listState = rememberLazyListState()

    LaunchedEffect(list?.groups) {
        listState.scrollToItem(0)
    }

    Scene(
        title = stringResource(id = R.string.settings_currency),
        snackbar = snackbar,
        onClose = onCancel,
    ) {
        SearchBar(query = query)
        LazyColumn(state = listState) {
            list?.let {
                gemGroupList(it) { item ->
                    when (item) {
                        is GemListItem.Currency -> onSelect(item.row)
                    }
                }
            }
        }
    }
}
