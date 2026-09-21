package com.gemwallet.android.testkit

import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.wallet.core.primitives.Asset

fun mockAssetInfoDataAggregate(asset: Asset = mockAsset(), pinned: Boolean = false) = AssetInfoDataAggregate(
    id = asset.id,
    asset = asset,
    title = asset.name,
    balance = "1.0 ${asset.symbol}",
    balanceEquivalent = "$1.00",
    isZeroBalance = false,
    price = null,
    pinned = pinned,
    balanceEnabled = true,
    accountAddress = mockAccount(chain = asset.id.chain).address,
)
