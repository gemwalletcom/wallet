package com.gemwallet.android.ext

import com.gemwallet.android.model.AssetPriceInfo
import com.gemwallet.android.model.AssetPriceValue
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import uniffi.gemstone.AssetPrice

fun AssetPrice.toAssetPriceInfo(currency: Currency): AssetPriceInfo = AssetPriceInfo(
    currency = currency,
    price = toPrimitives(),
)

fun List<AssetPrice>.toAssetPriceValue(asset: Asset, currency: Currency): AssetPriceValue = AssetPriceValue(
    asset = asset,
    price = firstOrNull { it.assetId == asset.id.toIdentifier() }?.toAssetPriceInfo(currency),
)
