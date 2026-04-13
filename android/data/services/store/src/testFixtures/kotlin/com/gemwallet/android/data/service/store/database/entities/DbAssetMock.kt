package com.gemwallet.android.data.service.store.database.entities

import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType

fun mockDbAsset(
    asset: Asset = Asset(
        id = AssetId(chain = com.wallet.core.primitives.Chain.Bitcoin),
        name = "Bitcoin",
        symbol = "BTC",
        decimals = 8,
        type = AssetType.NATIVE,
    ),
    isEnabled: Boolean = true,
    isBuyEnabled: Boolean = false,
    isSellEnabled: Boolean = false,
    isSwapEnabled: Boolean = false,
    isStakeEnabled: Boolean = false,
    stakingApr: Double? = null,
    rank: Int = 0,
    updatedAt: Long = 0,
) = DbAsset(
    id = asset.id.toIdentifier(),
    name = asset.name,
    symbol = asset.symbol,
    decimals = asset.decimals,
    type = asset.type,
    chain = asset.id.chain,
    isEnabled = isEnabled,
    isBuyEnabled = isBuyEnabled,
    isSellEnabled = isSellEnabled,
    isSwapEnabled = isSwapEnabled,
    isStakeEnabled = isStakeEnabled,
    stakingApr = stakingApr,
    rank = rank,
    updatedAt = updatedAt,
)
