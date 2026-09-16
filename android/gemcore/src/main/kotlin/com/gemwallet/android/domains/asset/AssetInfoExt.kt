package com.gemwallet.android.domains.asset

import uniffi.gemstone.GemSwapValue
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.toAssetPriceValue
import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.Chain
import java.math.BigDecimal
import java.math.BigInteger
import uniffi.gemstone.GemValueStyle

val AssetInfo.symbol: String
    get() = asset.symbol

val AssetInfo.decimals: Int
    get() = asset.decimals

val AssetInfo.title: String
    get() = asset.title

val AssetInfo.chain: Chain
    get() = asset.chain

val AssetInfo.availableBalanceAmount: String
    get() = ValueFormatter(style = GemValueStyle.AUTO)
        .string(balance.balance.available, decimals = asset.decimals)

fun AssetInfo.calculateFiat(value: BigInteger): BigDecimal = toAssetPriceValue().calculateFiat(value)

fun AssetInfo.calculateFiat(value: BigDecimal): BigDecimal = toAssetPriceValue().calculateFiat(value)

fun AssetInfo.formatFiat(value: BigDecimal): String = toAssetPriceValue().formatFiat(value)

fun AssetInfo.swapValue(value: BigInteger): GemSwapValue = toAssetPriceValue().swapValue(value)
