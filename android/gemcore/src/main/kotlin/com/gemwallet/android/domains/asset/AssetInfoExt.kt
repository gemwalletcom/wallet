package com.gemwallet.android.domains.asset

import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.model.toAssetPriceValue
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemValueStyle
import java.math.BigInteger

val AssetInfo.chain: Chain
    get() = asset.chain

fun AssetInfo.fiatEquivalent(value: BigInteger): String = toAssetPriceValue().fiatEquivalent(value)
