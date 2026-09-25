package com.gemwallet.android.features.asset.viewmodels.localization

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemPriceAlertToggle

@StringRes
fun GemPriceAlertToggle.toastRes(): Int = when (this) {
    GemPriceAlertToggle.ENABLED -> R.string.price_alerts_disabled_for
    GemPriceAlertToggle.DISABLED -> R.string.price_alerts_enabled_for
}
