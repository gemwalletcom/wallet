package com.gemwallet.android.ext

import com.gemwallet.android.model.AssetPriceInfo
import com.gemwallet.android.model.AssetPriceValue
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetPrice
import com.wallet.core.primitives.Currency
import uniffi.gemstone.GemAssetPrice

fun GemAssetPrice.toAssetPriceInfo(currency: Currency): AssetPriceInfo = AssetPriceInfo(
    currency = currency,
    price = AssetPrice(
        assetId = assetId.toAssetId()!!,
        price = price,
        priceChangePercentage24h = priceChangePercentage24h,
        updatedAt = updatedAt.secondsToMillis(),
    ),
)

fun List<GemAssetPrice>.toAssetPriceValue(asset: Asset, currency: Currency): AssetPriceValue = AssetPriceValue(
    asset = asset,
    price = firstOrNull { it.assetId == asset.id.toIdentifier() }?.toAssetPriceInfo(currency),
)
