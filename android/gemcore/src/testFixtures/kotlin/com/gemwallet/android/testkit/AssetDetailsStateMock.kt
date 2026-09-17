package com.gemwallet.android.testkit

import uniffi.gemstone.GemAssetDetailsState
import uniffi.gemstone.GemHeaderActions
import uniffi.gemstone.GemPriceAlertToggle

fun mockGemAssetDetailsState(
    showsBanners: Boolean = false,
    priceAlertsCount: UInt = 0u,
) = GemAssetDetailsState(
    isViewOnly = false,
    headerActions = GemHeaderActions.Buttons(emptyList()),
    showsBanners = showsBanners,
    showsManage = false,
    showsResources = false,
    showsPriceAlerts = priceAlertsCount > 0u,
    priceAlertsCount = priceAlertsCount,
    priceAlert = if (priceAlertsCount > 0u) GemPriceAlertToggle.ENABLED else GemPriceAlertToggle.DISABLED,
    showsEarn = false,
    emptyTransactionsAction = null,
)
