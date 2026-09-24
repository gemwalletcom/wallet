package com.gemwallet.android.domains.asset

import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.model.toAssetPriceValue
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemValueStyle
import java.math.BigInteger

val AssetInfo.symbol: String
    get() = asset.symbol

val AssetInfo.decimals: Int
    get() = asset.decimals

val AssetInfo.chain: Chain
    get() = asset.chain

fun AssetInfo.fiatEquivalent(value: BigInteger): String = toAssetPriceValue().fiatEquivalent(value)
