package com.gemwallet.android.features.activities.viewmodels.localization

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemSwapProgressStep
import uniffi.gemstone.GemTransactionParticipantRole

@StringRes
internal fun GemTransactionParticipantRole.stringRes(): Int = when (this) {
    GemTransactionParticipantRole.RECIPIENT -> R.string.transaction_recipient
    GemTransactionParticipantRole.SENDER -> R.string.transaction_sender
    GemTransactionParticipantRole.CONTRACT -> R.string.asset_contract
    GemTransactionParticipantRole.VALIDATOR -> R.string.stake_validator
    GemTransactionParticipantRole.PROVIDER -> R.string.common_provider
}

@StringRes
internal fun GemSwapProgressStep.stringRes(): Int? = when (this) {
    GemSwapProgressStep.COMPLETED -> R.string.transaction_status_completed
    GemSwapProgressStep.PENDING -> R.string.transaction_status_inprogress
    GemSwapProgressStep.WAITING -> null
    GemSwapProgressStep.FAILED -> R.string.transaction_status_failed
    GemSwapProgressStep.REVERTED -> R.string.transaction_status_reverted
    GemSwapProgressStep.REFUNDED -> R.string.transaction_status_refunded
}
