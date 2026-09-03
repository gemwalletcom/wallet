package com.gemwallet.android.features.settings.currency.presents

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
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
    val currentCurrency by viewModel.currency.collectAsStateWithLifecycle()
    val recommendedCurrencies by viewModel.recommendedCurrencies.collectAsStateWithLifecycle()
    val otherCurrencies by viewModel.otherCurrencies.collectAsStateWithLifecycle()

    Scene(
        title = stringResource(id = R.string.settings_currency),
        onClose = onCancel,
    ) {
        LazyColumn {
            item {
                SubheaderItem(R.string.common_recommended)
            }

            itemsIndexed(recommendedCurrencies) { index, item ->
                CurrencyItem(
                    currency = item,
                    selectedCurrency = currentCurrency,
                    listPosition = ListPosition.getPosition(index, recommendedCurrencies.size),
                    onSelect = {
                        viewModel.setCurrency(it)
                        onCancel()
                    }
                )
            }

            item {
                SubheaderItem(R.string.common_all)
            }
            itemsIndexed(otherCurrencies) { index, item ->
                CurrencyItem(
                    currency = item,
                    selectedCurrency = currentCurrency,
                    listPosition = ListPosition.getPosition(index, otherCurrencies.size),
                    onSelect = {
                        viewModel.setCurrency(it)
                        onCancel()
                    }
                )
            }
        }
    }
}