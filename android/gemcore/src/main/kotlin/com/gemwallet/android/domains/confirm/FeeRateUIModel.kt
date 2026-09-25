package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import com.wallet.core.primitives.FeePriority
import uniffi.gemstone.GemFeeRateRow

data class FeeRateUIModel(val row: GemFeeRateRow) {
    val priority: FeePriority = row.priority.toPrimitives()

    val fiatValue: String
        get() = row.amount?.fiat?.text().orEmpty()

    val emoji: String
        get() = when (priority) {
            FeePriority.Normal -> "\uD83D\uDC8E"
            FeePriority.Fast -> "\u26A1\uFE0F"
        }
}
