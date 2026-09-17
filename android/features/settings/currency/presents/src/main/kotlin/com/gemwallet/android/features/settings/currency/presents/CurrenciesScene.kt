package com.gemwallet.android.features.settings.currency.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.currency.viewmodels.CurrenciesViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.listSections
import com.gemwallet.android.ui.components.screen.Scene

@Composable
fun CurrenciesScene(
    onCancel: () -> Unit,
    viewModel: CurrenciesViewModel = hiltViewModel()
) {
    val sections by viewModel.sections.collectAsStateWithLifecycle()

    Scene(
        title = stringResource(id = R.string.settings_currency),
        onClose = onCancel,
    ) {
        LazyColumn {
            listSections(sections) { position, row ->
                ListItem(
                    model = row.model,
                    listPosition = position,
                    modifier = Modifier.clickable {
                        viewModel.setCurrency(row.currency)
                        onCancel()
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
