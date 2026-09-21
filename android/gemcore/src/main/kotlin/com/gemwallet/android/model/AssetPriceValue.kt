package com.gemwallet.android.model

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import uniffi.gemstone.GemSwapValue
import java.math.BigDecimal
import java.math.BigInteger

data class AssetPriceValue(val asset: Asset, val price: AssetPriceInfo?) {
    val currency: Currency? get() = price?.currency

    fun calculateFiat(value: BigInteger): BigDecimal = CryptoFiatConverter.fiatValue(Crypto(value), asset.decimals, price?.price?.price)?.toBigDecimal() ?: BigDecimal.ZERO

    fun calculateFiat(value: BigDecimal): BigDecimal = calculateFiat(Crypto(value, asset.decimals).atomicValue)

    fun formatFiat(value: BigDecimal): String {
        if (value <= BigDecimal.ZERO) return ""
        return price?.currency?.let { CurrencyFormatter(currency = it).string(value) } ?: ""
    }

    fun swapValue(value: BigInteger): GemSwapValue = GemSwapValue(
        value = value,
        decimals = asset.decimals.toUInt(),
        price = price?.price?.price,
    )
}

fun AssetInfo.toAssetPriceValue(): AssetPriceValue = AssetPriceValue(asset, price)
