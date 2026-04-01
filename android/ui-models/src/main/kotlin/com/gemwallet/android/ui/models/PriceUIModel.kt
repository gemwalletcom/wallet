package com.gemwallet.android.ui.models

import com.gemwallet.android.domains.price.PriceState
import com.gemwallet.android.domains.price.toPriceState

interface PriceUIModel : FiatFormattedUIModel, PercentageFormattedUIModel {
    val state: PriceState
        get() = percentage.toPriceState()
}
