package com.gemwallet.android.domains.asset

import uniffi.gemstone.GemSwapValue
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.toAssetPriceValue
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.model.CurrencyFormatter
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.StakeChain
import java.math.BigDecimal
import java.math.BigInteger

val AssetInfo.symbol: String
    get() = asset.symbol

val AssetInfo.decimals: Int
    get() = asset.decimals

val AssetInfo.title: String
    get() = asset.title

val AssetInfo.stakeChain: StakeChain? // TODO: Out to StakeExt
    get() = asset.stakeChain

val AssetInfo.chain: Chain
    get() = asset.chain

val AssetInfo.availableBalanceFormatted: String // TODO: Out to BalanceExt
    get() = ValueFormatter(style = ValueFormatter.Style.Auto)
        .string(balance.balance.available, balance.asset)

val AssetInfo.availableBalanceAmount: String
    get() = ValueFormatter(style = ValueFormatter.Style.Auto)
        .string(balance.balance.available, decimals = asset.decimals)

fun AssetInfo.calculateFiat(value: BigInteger): BigDecimal = toAssetPriceValue().calculateFiat(value)

fun AssetInfo.calculateFiat(value: BigDecimal): BigDecimal = toAssetPriceValue().calculateFiat(value)

fun AssetInfo.formatFiat(value: BigDecimal): String = toAssetPriceValue().formatFiat(value)

fun AssetInfo.swapValue(value: BigInteger): GemSwapValue = toAssetPriceValue().swapValue(value)
