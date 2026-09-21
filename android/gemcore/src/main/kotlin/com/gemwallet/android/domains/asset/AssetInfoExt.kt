package com.gemwallet.android.domains.asset

import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.model.toAssetPriceValue
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemSwapValue
import uniffi.gemstone.GemValueStyle
import java.math.BigDecimal
import java.math.BigInteger

val AssetInfo.symbol: String
    get() = asset.symbol

val AssetInfo.decimals: Int
    get() = asset.decimals

val AssetInfo.chain: Chain
    get() = asset.chain

fun AssetInfo.calculateFiat(value: BigInteger): BigDecimal = toAssetPriceValue().calculateFiat(value)

fun AssetInfo.calculateFiat(value: BigDecimal): BigDecimal = toAssetPriceValue().calculateFiat(value)

fun AssetInfo.formatFiat(value: BigDecimal): String = toAssetPriceValue().formatFiat(value)

fun AssetInfo.swapValue(value: BigInteger): GemSwapValue = toAssetPriceValue().swapValue(value)
