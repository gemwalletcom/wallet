package com.gemwallet.android.testkit

import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.AssetMarket

fun mockAssetLink() = AssetLink(
    name = "website",
    url = "https://bitcoin.org",
)

fun mockAssetMarket(
    marketCap: Double? = null,
) = AssetMarket(
    marketCap = marketCap,
    marketCapFdv = null,
    marketCapRank = null,
    totalVolume = null,
    circulatingSupply = null,
    totalSupply = null,
    maxSupply = null,
    allTimeHighValue = null,
    allTimeLowValue = null,
)
