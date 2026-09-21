package com.gemwallet.android.features.settings.price_alerts.viewmodels.localization

import android.content.Context
import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemPriceAlertPrompt
import uniffi.gemstone.GemPriceAlertSectionKind

@StringRes
internal fun GemPriceAlertPrompt.stringRes(): Int = when (this) {
    GemPriceAlertPrompt.TARGET_PRICE -> R.string.price_alerts_set_alert_set_target_price
    GemPriceAlertPrompt.PRICE_OVER -> R.string.price_alerts_set_alert_price_over
    GemPriceAlertPrompt.PRICE_UNDER -> R.string.price_alerts_set_alert_price_under
    GemPriceAlertPrompt.INCREASES_BY -> R.string.price_alerts_set_alert_price_increases_by
    GemPriceAlertPrompt.DECREASES_BY -> R.string.price_alerts_set_alert_price_decreases_by
}

internal fun GemPriceAlertSectionKind.title(): String? = when (this) {
    GemPriceAlertSectionKind.Auto -> null
    is GemPriceAlertSectionKind.Asset -> name
}

internal fun GemPriceAlertSectionKind.footer(context: Context): String? = when (this) {
    GemPriceAlertSectionKind.Auto -> context.getString(R.string.price_alerts_auto_footer)
    is GemPriceAlertSectionKind.Asset -> null
}
