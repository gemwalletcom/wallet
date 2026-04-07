package com.gemwallet.android.features.settings.price_alerts.presents

import android.content.res.Resources
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.platform.LocalContext
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.price_alerts.viewmodels.PriceAlertTargetViewModel
import com.gemwallet.android.features.settings.price_alerts.viewmodels.models.PriceAlertConfirmResult
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.PriceAlertDirection
import com.wallet.core.primitives.PriceAlertNotificationType

@Composable
fun PriceAlertTargetNavScreen(
    onCancel: () -> Unit,
    onComplete: (String) -> Unit = { onCancel() },
    viewModel: PriceAlertTargetViewModel = hiltViewModel(),
) {
    val resources = LocalContext.current.resources
    val currency by viewModel.currency.collectAsStateWithLifecycle()
    val currentPriceFormatted by viewModel.currentPrice.collectAsStateWithLifecycle()
    val currentPriceValue by viewModel.currentPriceValue.collectAsStateWithLifecycle()
    val type by viewModel.type.collectAsStateWithLifecycle()
    val direction by viewModel.direction.collectAsStateWithLifecycle()
    val error by viewModel.error.collectAsStateWithLifecycle()

    PriceAlertTargetScene(
        value = viewModel.value,
        type = type,
        direction = direction,
        currency = currency,
        currentPriceValue = currentPriceValue,
        currentPriceFormatted = currentPriceFormatted,
        error = error,
        onType = viewModel::onType,
        onDirection = viewModel::onDirection,
        onConfirm = {
            val result = viewModel.onConfirm()
            if (result != null) {
                onComplete(result.toMessage(resources))
            } else {
                onCancel()
            }
        },
        onCancel = onCancel,
    )
}

private fun PriceAlertConfirmResult.toMessage(resources: Resources): String {
    val directionTitle = when (type) {
        PriceAlertNotificationType.Price -> when (direction) {
            PriceAlertDirection.Up -> R.string.price_alerts_set_alert_price_over
            PriceAlertDirection.Down -> R.string.price_alerts_set_alert_price_under
        }
        PriceAlertNotificationType.PricePercentChange -> when (direction) {
            PriceAlertDirection.Up -> R.string.price_alerts_set_alert_price_increases_by
            PriceAlertDirection.Down -> R.string.price_alerts_set_alert_price_decreases_by
        }
        PriceAlertNotificationType.Auto -> return ""
    }
    val message = "${resources.getString(directionTitle).lowercase()} $amount"
    return resources.getString(R.string.price_alerts_added_for, message)
}
