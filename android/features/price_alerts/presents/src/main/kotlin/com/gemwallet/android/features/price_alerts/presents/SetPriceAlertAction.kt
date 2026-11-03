package com.gemwallet.android.features.price_alerts.presents

import com.wallet.core.primitives.PriceAlertDirection
import com.wallet.core.primitives.PriceAlertNotificationType

internal sealed interface SetPriceAlertAction {
    data class SelectType(val type: PriceAlertNotificationType) : SetPriceAlertAction
    data class SelectDirection(val direction: PriceAlertDirection) : SetPriceAlertAction
    data object Confirm : SetPriceAlertAction
    data object Cancel : SetPriceAlertAction
}
