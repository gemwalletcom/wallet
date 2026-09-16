package com.gemwallet.android.ui.style

import androidx.annotation.DrawableRes
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.pendingColor
import uniffi.gemstone.GemDelegationTone
import uniffi.gemstone.GemHeaderButtonKind
import uniffi.gemstone.GemTransactionStateTone
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.GemVerificationLevel
import uniffi.gemstone.WalletConnectionVerificationStatus
import uniffi.gemstone.verificationLevel

@Composable
fun GemHeaderButtonKind.icon(): ImageVector = when (this) {
    GemHeaderButtonKind.SEND -> AppIcons.Send
    GemHeaderButtonKind.RECEIVE -> AppIcons.Receive
    GemHeaderButtonKind.BUY -> AppIcons.Buy
    GemHeaderButtonKind.SWAP -> AppIcons.SwapVert
    GemHeaderButtonKind.DEPOSIT -> AppIcons.Deposit
    GemHeaderButtonKind.WITHDRAW -> AppIcons.Withdraw
    GemHeaderButtonKind.MORE -> AppIcons.MoreVert
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

@Composable
fun GemValueTone.color(): Color = when (this) {
    GemValueTone.PLAIN -> MaterialTheme.colorScheme.onSurface
    GemValueTone.NEUTRAL -> MaterialTheme.colorScheme.secondary
    GemValueTone.POSITIVE -> MaterialTheme.colorScheme.tertiary
    GemValueTone.NEGATIVE -> MaterialTheme.colorScheme.error
}

@Composable
fun GemDelegationTone.color(): Color = when (this) {
    GemDelegationTone.POSITIVE -> MaterialTheme.colorScheme.tertiary
    GemDelegationTone.PENDING -> pendingColor
    GemDelegationTone.NEGATIVE -> MaterialTheme.colorScheme.error
}

@Composable
fun WalletConnectionVerificationStatus.icon(): ImageVector = when (verificationLevel(this)) {
    GemVerificationLevel.VERIFIED -> AppIcons.Verified
    GemVerificationLevel.UNVERIFIED, GemVerificationLevel.SUSPICIOUS -> AppIcons.Warning
}

@Composable
fun WalletConnectionVerificationStatus.color(): Color = when (verificationLevel(this)) {
    GemVerificationLevel.VERIFIED -> MaterialTheme.colorScheme.tertiary
    GemVerificationLevel.UNVERIFIED -> pendingColor
    GemVerificationLevel.SUSPICIOUS -> MaterialTheme.colorScheme.error
}
