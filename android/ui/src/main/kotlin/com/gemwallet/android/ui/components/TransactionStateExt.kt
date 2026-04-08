package com.gemwallet.android.ui.components

import androidx.annotation.DrawableRes
import androidx.annotation.StringRes
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.theme.pendingColor
import com.wallet.core.primitives.TransactionState

enum class TransactionStateTone {
    Pending,
    Success,
    Error,
}

@StringRes
fun TransactionState.statusLabelRes(): Int = when (this) {
    TransactionState.Pending,
    TransactionState.InTransit -> R.string.transaction_status_pending
    TransactionState.Confirmed -> R.string.transaction_status_confirmed
    TransactionState.Failed -> R.string.transaction_status_failed
    TransactionState.Reverted -> R.string.transaction_status_reverted
}

@StringRes
fun TransactionState.statusInfoDescriptionRes(): Int = when (this) {
    TransactionState.Pending -> R.string.info_transaction_pending_description
    TransactionState.Confirmed -> R.string.info_transaction_success_description
    TransactionState.Failed,
    TransactionState.Reverted,
    TransactionState.InTransit -> R.string.info_transaction_error_description
}

@DrawableRes
fun TransactionState.statusBadgeIconRes(): Int = when (this) {
    TransactionState.Pending,
    TransactionState.InTransit -> R.drawable.transaction_state_pending
    TransactionState.Confirmed -> R.drawable.transaction_state_success
    TransactionState.Failed,
    TransactionState.Reverted -> R.drawable.transaction_state_error
}

fun TransactionState.statusTone(): TransactionStateTone = when (this) {
    TransactionState.Pending,
    TransactionState.InTransit -> TransactionStateTone.Pending
    TransactionState.Confirmed -> TransactionStateTone.Success
    TransactionState.Failed,
    TransactionState.Reverted -> TransactionStateTone.Error
}

fun TransactionState.showsStatusBadge(): Boolean = this != TransactionState.Confirmed

fun TransactionState.showsStatusProgress(): Boolean = this == TransactionState.Pending

@Composable
fun TransactionState.statusColor(): Color = when (statusTone()) {
    TransactionStateTone.Pending -> pendingColor
    TransactionStateTone.Success -> MaterialTheme.colorScheme.tertiary
    TransactionStateTone.Error -> MaterialTheme.colorScheme.error
}
