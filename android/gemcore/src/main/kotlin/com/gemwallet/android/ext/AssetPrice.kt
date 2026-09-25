package com.gemwallet.android.ext

import com.gemwallet.android.model.AssetPriceInfo
import com.wallet.core.primitives.Currency
import uniffi.gemstone.AssetPrice

fun AssetPrice.toAssetPriceInfo(currency: Currency): AssetPriceInfo = AssetPriceInfo(
    currency = currency,
    price = toPrimitives(),
)
