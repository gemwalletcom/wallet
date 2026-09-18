package com.gemwallet.android.testkit

import uniffi.gemstone.GemAssetDetailsState
import uniffi.gemstone.GemHeaderActions
import uniffi.gemstone.GemPriceAlertToggle

fun mockGemAssetDetailsState(
    showsBanners: Boolean = false,
    priceAlertsCount: Int = 0,
) = GemAssetDetailsState(
    isViewOnly = false,
    headerActions = GemHeaderActions.Buttons(emptyList()),
    showsBanners = showsBanners,
    showsManage = false,
    showsResources = false,
    showsPriceAlerts = priceAlertsCount > 0,
    priceAlertsCountText = priceAlertsCount.toString(),
    priceAlert = if (priceAlertsCount > 0) GemPriceAlertToggle.ENABLED else GemPriceAlertToggle.DISABLED,
    showsEarn = false,
    emptyTransactionsAction = null,
)
