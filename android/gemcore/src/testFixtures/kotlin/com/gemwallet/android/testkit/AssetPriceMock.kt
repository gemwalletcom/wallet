package com.gemwallet.android.testkit

import com.gemwallet.android.model.AssetPriceInfo
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetPrice
import com.wallet.core.primitives.Currency

fun mockAssetPrice(
    assetId: AssetId = mockAssetId(),
    price: Double = 50000.0,
    priceChangePercentage24h: Double = 0.0,
    updatedAt: Long = 0L,
) = AssetPrice(
    assetId = assetId,
    price = price,
    priceChangePercentage24h = priceChangePercentage24h,
    updatedAt = updatedAt,
)

fun mockAssetPriceInfo(
    price: Double = 50000.0,
    priceChangePercentage24h: Double = 0.0,
    updatedAt: Long = 0L,
    currency: Currency = Currency.USD,
) = AssetPriceInfo(
    currency = currency,
    price = mockAssetPrice(
        price = price,
        priceChangePercentage24h = priceChangePercentage24h,
        updatedAt = updatedAt,
    ),
)
