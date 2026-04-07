package com.gemwallet.android.features.settings.price_alerts.viewmodels.models

import com.wallet.core.primitives.PriceAlertDirection
import com.wallet.core.primitives.PriceAlertNotificationType

data class PriceAlertConfirmResult(
    val type: PriceAlertNotificationType,
    val direction: PriceAlertDirection,
    val amount: String,
)
