package com.gemwallet.android.ui.components

import androidx.annotation.DrawableRes
import androidx.annotation.StringRes
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.theme.pendingColor
import uniffi.gemstone.GemTransactionStateTone
import com.wallet.core.primitives.TransactionState

@StringRes
fun TransactionState.statusLabelRes(): Int = when (this) {
    TransactionState.Pending,
    TransactionState.InTransit -> R.string.transaction_status_pending
    TransactionState.Confirmed -> R.string.transaction_status_confirmed
    TransactionState.Failed -> R.string.transaction_status_failed
    TransactionState.Reverted -> R.string.transaction_status_reverted
    TransactionState.Refunded -> R.string.transaction_status_refunded
}

@StringRes
fun GemTransactionStateTone.infoDescriptionRes(): Int = when (this) {
    GemTransactionStateTone.PENDING -> R.string.info_transaction_pending_description
    GemTransactionStateTone.SUCCESS -> R.string.info_transaction_success_description
    GemTransactionStateTone.ERROR,
    GemTransactionStateTone.REFUNDED -> R.string.info_transaction_error_description
}

@DrawableRes
fun GemTransactionStateTone.badgeIconRes(): Int = when (this) {
    GemTransactionStateTone.PENDING -> R.drawable.transaction_state_pending
    GemTransactionStateTone.SUCCESS -> R.drawable.transaction_state_success
    GemTransactionStateTone.ERROR,
    GemTransactionStateTone.REFUNDED -> R.drawable.transaction_state_error
}

@Composable
fun GemTransactionStateTone.color(): Color = when (this) {
    GemTransactionStateTone.PENDING,
    GemTransactionStateTone.REFUNDED -> pendingColor
    GemTransactionStateTone.SUCCESS -> MaterialTheme.colorScheme.tertiary
    GemTransactionStateTone.ERROR -> MaterialTheme.colorScheme.error
}
