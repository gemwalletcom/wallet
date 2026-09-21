package com.gemwallet.android.features.asset.viewmodels.style

import com.gemwallet.android.ui.components.list_item.ListItemSymbol
import uniffi.gemstone.GemPriceAlertToggle

fun GemPriceAlertToggle.symbol(): ListItemSymbol = when (this) {
    GemPriceAlertToggle.ENABLED -> ListItemSymbol.Notifications
    GemPriceAlertToggle.DISABLED -> ListItemSymbol.NotificationsOutlined
}
