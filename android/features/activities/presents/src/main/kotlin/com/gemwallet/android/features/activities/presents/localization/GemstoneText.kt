package com.gemwallet.android.features.activities.presents.localization

import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
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

@StringRes
internal fun TransactionDetailsValue.Destination.stringRes(): Int = when (this) {
    is TransactionDetailsValue.Destination.Recipient -> R.string.transaction_recipient
    is TransactionDetailsValue.Destination.Sender -> R.string.transaction_sender
    is TransactionDetailsValue.Destination.Contract -> R.string.asset_contract
    is TransactionDetailsValue.Destination.Validator -> R.string.stake_validator
    is TransactionDetailsValue.Destination.ProviderAddress -> R.string.common_provider
    is TransactionDetailsValue.Destination.Provider -> R.string.common_provider
}
