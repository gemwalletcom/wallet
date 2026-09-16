package com.gemwallet.android.features.asset.presents.style

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector
import com.gemwallet.android.ui.icons.AppIcons
import uniffi.gemstone.GemPriceAlertToggle

@Composable
internal fun GemPriceAlertToggle.icon(): ImageVector = when (this) {
    GemPriceAlertToggle.ENABLED -> AppIcons.Notifications
    GemPriceAlertToggle.DISABLED -> AppIcons.NotificationsOutlined
}
