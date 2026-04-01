package com.gemwallet.android.ui.models

import com.gemwallet.android.domains.percentage.formatAsPercentage

interface PercentageFormattedUIModel {
    val percentage: Double?

    val percentageFormatted: String
        get() = percentage.formatAsPercentage()
}
