package com.gemwallet.android.model

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import uniffi.gemstone.fiatEquivalent
import java.math.BigInteger

data class AssetPriceValue(val asset: Asset, val price: AssetPriceInfo?) {
    val currency: Currency? get() = price?.currency

    fun fiatEquivalent(value: BigInteger): String {
        val price = price ?: return ""
        return fiatEquivalent(asset.toGem(), value, price.price.price, price.currency.toGem())?.text().orEmpty()
    }
}

fun AssetInfo.toAssetPriceValue(): AssetPriceValue = AssetPriceValue(asset, price)
