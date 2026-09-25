package com.gemwallet.android.features.settings.price_alerts.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.price_alerts.viewmodels.PriceAlertTargetViewModel
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.gemwallet.android.ui.style.textStyle
import uniffi.gemstone.GemValueTone

@Composable
fun PriceAlertTargetNavScreen(onCancel: () -> Unit, onComplete: (String) -> Unit = { onCancel() }, viewModel: PriceAlertTargetViewModel = hiltViewModel()) {
    val currency = viewModel.currency
    val currentPriceFormatted by viewModel.currentPrice.collectAsStateWithLifecycle()
    val type by viewModel.type.collectAsStateWithLifecycle()
    val direction by viewModel.direction.collectAsStateWithLifecycle()
    val prompt by viewModel.prompt.collectAsStateWithLifecycle()
    val priceSuggestions by viewModel.priceSuggestions.collectAsStateWithLifecycle()
    val percentageSuggestions by viewModel.percentageSuggestions.collectAsStateWithLifecycle()
    val asset by viewModel.asset.collectAsStateWithLifecycle()
    val priceChange by viewModel.priceChange.collectAsStateWithLifecycle()
    val buttonState by viewModel.buttonState.collectAsStateWithLifecycle()
    val error by viewModel.error.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = error, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)

    PriceAlertTargetScene(
        value = viewModel.value,
        type = type,
        direction = direction,
        prompt = prompt,
        currency = currency,
        currentPriceFormatted = currentPriceFormatted,
        priceSuggestions = priceSuggestions,
        percentageSuggestions = percentageSuggestions,
        asset = asset,
        assetPriceFormatted = currentPriceFormatted,
        assetPriceChangeFormatted = priceChange?.text().orEmpty(),
        assetValueStyle = (priceChange?.tone ?: GemValueTone.NEUTRAL).textStyle(),
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
