package com.gemwallet.android.testkit

import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.AssetMarket
import com.wallet.core.primitives.ChartValuePercentage

fun mockAssetLink(
    name: String = "website",
    url: String = "https://bitcoin.org",
) = AssetLink(
    name = name,
    url = url,
)

fun mockAssetMarket(
    marketCap: Double? = null,
    marketCapFdv: Double? = null,
    marketCapRank: Int? = null,
    totalVolume: Double? = null,
    circulatingSupply: Double? = null,
    totalSupply: Double? = null,
    maxSupply: Double? = null,
    allTimeHighValue: ChartValuePercentage? = null,
    allTimeLowValue: ChartValuePercentage? = null,
) = AssetMarket(
    marketCap = marketCap,
    marketCapFdv = marketCapFdv,
    marketCapRank = marketCapRank,
    totalVolume = totalVolume,
    circulatingSupply = circulatingSupply,
    totalSupply = totalSupply,
    maxSupply = maxSupply,
    allTimeHighValue = allTimeHighValue,
    allTimeLowValue = allTimeLowValue,
)
