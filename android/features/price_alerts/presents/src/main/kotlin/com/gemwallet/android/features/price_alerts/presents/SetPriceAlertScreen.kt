package com.gemwallet.android.features.price_alerts.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.price_alerts.viewmodels.SetPriceAlertViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.rememberSnackbarState

@Composable
fun SetPriceAlertScreen(onCancel: () -> Unit, onComplete: (String) -> Unit = { onCancel() }, viewModel: SetPriceAlertViewModel = hiltViewModel()) {
    val currentPriceText by viewModel.currentPriceText.collectAsStateWithLifecycle()
    val input by viewModel.input.collectAsStateWithLifecycle()
    val type by viewModel.type.collectAsStateWithLifecycle()
    val direction by viewModel.direction.collectAsStateWithLifecycle()
    val prompt by viewModel.prompt.collectAsStateWithLifecycle()
    val priceSuggestions by viewModel.priceSuggestions.collectAsStateWithLifecycle()
    val percentageSuggestions by viewModel.percentageSuggestions.collectAsStateWithLifecycle()
    val assetRow by viewModel.assetRow.collectAsStateWithLifecycle()
    val buttonState by viewModel.buttonState.collectAsStateWithLifecycle()
    val error by viewModel.error.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = error, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)

    SetPriceAlertScene(
        value = viewModel.value,
        type = type,
        direction = direction,
        prompt = prompt,
        input = input,
        currentPriceText = currentPriceText,
        priceSuggestions = priceSuggestions,
        percentageSuggestions = percentageSuggestions,
        assetRow = assetRow,
        buttonState = buttonState,
        snackbar = snackbar,
        onType = viewModel::onType,
        onDirection = viewModel::onDirection,
        onConfirm = {
            viewModel.onConfirm(onComplete)
        },
        onCancel = onCancel,
    )
}
