package com.gemwallet.android.ui.components

import androidx.annotation.StringRes
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Verified
import androidx.compose.material.icons.outlined.Warning
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.theme.pendingColor
import uniffi.gemstone.WalletConnectionVerificationStatus

@StringRes
fun WalletConnectionVerificationStatus.titleRes(): Int = when (this) {
    WalletConnectionVerificationStatus.VERIFIED -> R.string.asset_verification_verified
    WalletConnectionVerificationStatus.UNKNOWN -> R.string.asset_verification_unverified
    WalletConnectionVerificationStatus.INVALID,
    WalletConnectionVerificationStatus.MALICIOUS -> R.string.asset_verification_suspicious
}

fun WalletConnectionVerificationStatus.icon(): ImageVector = when (this) {
    WalletConnectionVerificationStatus.VERIFIED -> Icons.Outlined.Verified
    else -> Icons.Outlined.Warning
}

@Composable
fun WalletConnectionVerificationStatus.color(): Color = when (this) {
    WalletConnectionVerificationStatus.VERIFIED -> MaterialTheme.colorScheme.tertiary
    WalletConnectionVerificationStatus.UNKNOWN -> pendingColor
    WalletConnectionVerificationStatus.INVALID,
    WalletConnectionVerificationStatus.MALICIOUS -> MaterialTheme.colorScheme.error
}
