package com.gemwallet.android.ui.models

import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.domains.price.toValueDirection

interface PriceUIModel : FiatFormattedUIModel, PercentageFormattedUIModel {
    val state: ValueDirection
        get() = percentage.toValueDirection()
}
