package com.gemwallet.android.testkit

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetFull
import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.AssetMarket
import com.wallet.core.primitives.AssetProperties
import com.wallet.core.primitives.AssetScore
import com.wallet.core.primitives.ChartValuePercentage
import com.wallet.core.primitives.PerpetualBasic
import com.wallet.core.primitives.Price

fun mockAssetFull(
    asset: Asset = mockAsset(),
    properties: AssetProperties = mockAssetProperties(),
    score: AssetScore = AssetScore(rank = 100),
    tags: List<String> = emptyList(),
    links: List<AssetLink> = emptyList(),
    perpetuals: List<PerpetualBasic> = emptyList(),
    price: Price? = null,
    market: AssetMarket? = null,
) = AssetFull(
    asset = asset,
    properties = properties,
    score = score,
    tags = tags,
    links = links,
    perpetuals = perpetuals,
    price = price,
    market = market,
)

fun mockAssetLink(
    name: String = "website",
    url: String = "https://bitcoin.org",
) = AssetLink(
    name = name,
    url = url,
)

fun mockAssetProperties(
    isEnabled: Boolean = true,
    isBuyable: Boolean = true,
    isSellable: Boolean = true,
    isSwapable: Boolean = true,
    isStakeable: Boolean = false,
    stakingApr: Double? = null,
    isEarnable: Boolean = false,
    earnApr: Double? = null,
    hasImage: Boolean = true,
) = AssetProperties(
    isEnabled = isEnabled,
    isBuyable = isBuyable,
    isSellable = isSellable,
    isSwapable = isSwapable,
    isStakeable = isStakeable,
    stakingApr = stakingApr,
    isEarnable = isEarnable,
    earnApr = earnApr,
    hasImage = hasImage,
)

fun mockPrice(
    price: Double = 50000.0,
    priceChangePercentage24h: Double = 0.0,
    updatedAt: Long = 0L,
) = Price(
    price = price,
    priceChangePercentage24h = priceChangePercentage24h,
    updatedAt = updatedAt,
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

fun mockChartValuePercentage(
    date: Long = 0L,
    value: Float = 0.0f,
    percentage: Float = 0.0f,
) = ChartValuePercentage(
    date = date,
    value = value,
    percentage = percentage,
)
