package com.gemwallet.android.features.settings.presents.currency

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.viewmodels.currency.CurrencyViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.rememberSnackbarState

@Composable
fun CurrencyScreen(onCancel: () -> Unit, viewModel: CurrencyViewModel = hiltViewModel()) {
    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val error by viewModel.error.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = error, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)

    CurrencyScene(
        sections = sections,
        query = viewModel.query,
        snackbar = snackbar,
        onSelect = { viewModel.setCurrency(it, onCancel) },
        onCancel = onCancel,
    )
}
