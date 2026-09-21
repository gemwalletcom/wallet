package com.gemwallet.android.testkit

import uniffi.gemstone.GemAssetDetailsState
import uniffi.gemstone.GemHeaderActions
import uniffi.gemstone.GemPriceAlertToggle

fun mockGemAssetDetailsState(showsBanners: Boolean = false, priceAlertsCount: Int = 0) = GemAssetDetailsState(
    isViewOnly = false,
    headerActions = GemHeaderActions.Buttons(emptyList()),
    showsBanners = showsBanners,
    priceAlert = if (priceAlertsCount > 0) GemPriceAlertToggle.ENABLED else GemPriceAlertToggle.DISABLED,
    emptyTransactionsAction = null,
)
