package com.gemwallet.android.testkit

import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.asset.icon
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemAssetListRow
import uniffi.gemstone.GemAssetRowText
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.GemPriceRow

fun mockAssetInfoDataAggregate(asset: Asset = mockAsset(), pinned: Boolean = false) = AssetInfoDataAggregate(
    asset = asset,
    row = GemAssetListRow(
        icon = asset.id.icon(),
        text = GemAssetRowText(title = asset.name, symbol = null, network = null),
        price = GemPriceRow(price = null, change = null),
        amount = mockFormattedNumber(1.0, unit = GemNumberUnit.Symbol(symbol = asset.symbol)),
        fiat = mockFormattedNumber(1.0),
    ),
    hideBalance = false,
    pinned = pinned,
    balanceEnabled = true,
    accountAddress = mockAccount(chain = asset.id.chain).address,
)
