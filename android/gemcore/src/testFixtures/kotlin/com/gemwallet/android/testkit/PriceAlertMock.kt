package com.gemwallet.android.testkit

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertDirection

fun mockPriceAlert(
    assetId: AssetId = mockAssetId(),
    currency: String = "USD",
    price: Double? = null,
    pricePercentChange: Double? = null,
    priceDirection: PriceAlertDirection? = null,
) = PriceAlert(
    assetId = assetId,
    currency = currency,
    price = price,
    pricePercentChange = pricePercentChange,
    priceDirection = priceDirection,
    lastNotifiedAt = null,
)
