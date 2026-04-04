package com.gemwallet.android.testkit

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetScore

fun mockAssetBasic(
    asset: Asset = mockAsset(),
    rank: Int = 100,
) = AssetBasic(
    asset = asset,
    properties = mockAssetProperties(),
    score = AssetScore(rank = rank),
)
