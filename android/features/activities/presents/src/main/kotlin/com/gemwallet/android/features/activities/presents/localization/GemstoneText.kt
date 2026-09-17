package com.gemwallet.android.features.activities.presents.localization

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemSwapProgressStep

@StringRes
internal fun GemSwapProgressStep.stringRes(): Int? = when (this) {
    GemSwapProgressStep.COMPLETED -> R.string.transaction_status_completed
    GemSwapProgressStep.PENDING -> R.string.transaction_status_inprogress
    GemSwapProgressStep.WAITING -> null
    GemSwapProgressStep.FAILED -> R.string.transaction_status_failed
    GemSwapProgressStep.REVERTED -> R.string.transaction_status_reverted
    GemSwapProgressStep.REFUNDED -> R.string.transaction_status_refunded
}
