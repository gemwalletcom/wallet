package com.gemwallet.android.features.asset.presents.details.components

import androidx.annotation.DrawableRes
import androidx.annotation.StringRes
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.theme.pendingColor

@StringRes
internal fun AssetVerification.labelRes(): Int = when (this) {
    AssetVerification.Suspicious -> R.string.asset_verification_suspicious
    AssetVerification.Unverified -> R.string.asset_verification_unverified
}

@DrawableRes
internal fun AssetVerification.badgeIconRes(): Int = when (this) {
    AssetVerification.Suspicious -> R.drawable.suspicious
    AssetVerification.Unverified -> R.drawable.unverified
}

internal fun AssetVerification.infoSheetEntity(): InfoSheetEntity = when (this) {
    AssetVerification.Suspicious -> InfoSheetEntity.AssetStatusSuspiciousInfo
    AssetVerification.Unverified -> InfoSheetEntity.AssetStatusUnverifiedInfo
}

@Composable
internal fun AssetVerification.color(): Color = when (this) {
    AssetVerification.Suspicious -> MaterialTheme.colorScheme.error
    AssetVerification.Unverified -> pendingColor
}
