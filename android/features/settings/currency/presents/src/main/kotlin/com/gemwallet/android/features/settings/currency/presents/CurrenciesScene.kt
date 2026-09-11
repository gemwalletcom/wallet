package com.gemwallet.android.features.settings.currency.presents

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ext.toCurrency
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.features.settings.currency.presents.components.CurrencyItem
import com.gemwallet.android.features.settings.currency.viewmodels.CurrenciesViewModel

@Composable
fun CurrenciesScene(
    onCancel: () -> Unit,
    viewModel: CurrenciesViewModel = hiltViewModel()
) {
    val currencies by viewModel.currencies.collectAsStateWithLifecycle()
    val recommended = currencies?.recommended.orEmpty()
    val other = currencies?.other.orEmpty()
    val selected = currencies?.selected?.currency

    Scene(
        title = stringResource(id = R.string.settings_currency),
        onClose = onCancel,
    ) {
        LazyColumn {
            item {
                SubheaderItem(R.string.common_recommended)
            }

            itemsIndexed(recommended) { index, item ->
                CurrencyItem(
                    row = item,
                    isSelected = item.currency == selected,
                    listPosition = ListPosition.getPosition(index, recommended.size),
                    onSelect = {
                        viewModel.setCurrency(it.currency.toCurrency())
                        onCancel()
                    }
                )
            }

            item {
                SubheaderItem(R.string.common_all)
            }
            itemsIndexed(other) { index, item ->
                CurrencyItem(
                    row = item,
                    isSelected = item.currency == selected,
                    listPosition = ListPosition.getPosition(index, other.size),
                    onSelect = {
                        viewModel.setCurrency(it.currency.toCurrency())
                        onCancel()
                    }
                )
            }
        }
    }
}
