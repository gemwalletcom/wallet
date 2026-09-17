package com.gemwallet.android.features.settings.price_alerts.presents.localization

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemPriceAlertLabel
import uniffi.gemstone.GemPriceAlertText

private const val EMPTY = "-"

@Composable
internal fun GemPriceAlertText.string(): String = when (this) {
    is GemPriceAlertText.Empty -> EMPTY
    is GemPriceAlertText.Number -> value.text()
    is GemPriceAlertText.Label -> label.string()
}

@Composable
internal fun GemPriceAlertLabel.string(): String = when (this) {
    GemPriceAlertLabel.OVER -> stringResource(R.string.price_alerts_direction_over)
    GemPriceAlertLabel.UNDER -> stringResource(R.string.price_alerts_direction_under)
    GemPriceAlertLabel.INCREASES_BY -> stringResource(R.string.price_alerts_direction_increases_by)
    GemPriceAlertLabel.DECREASES_BY -> stringResource(R.string.price_alerts_direction_decreases_by)
}
