package com.gemwallet.android.testkit

import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.asset.icon
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemAssetItemRow
import uniffi.gemstone.GemAssetItemTrailing
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.GemRowText
import uniffi.gemstone.GemValueTone

fun mockAssetInfoDataAggregate(asset: Asset = mockAsset(), pinned: Boolean = false) = AssetInfoDataAggregate(
    asset = asset,
    row = GemAssetItemRow(
        icon = asset.id.icon(),
        title = asset.name,
        titleExtra = null,
        subtitle = null,
        subtitleExtra = null,
        trailing = GemAssetItemTrailing.Value(
            value = GemRowText(GemLocalizedText.Number(mockFormattedNumber(1.0, unit = GemNumberUnit.Symbol(symbol = asset.symbol))), GemValueTone.PLAIN),
            extra = GemRowText(GemLocalizedText.Number(mockFormattedNumber(1.0)), GemValueTone.NEUTRAL),
        ),
        masksBalance = true,
    ),
    hideBalance = false,
    pinned = pinned,
    balanceEnabled = true,
    accountAddress = mockAccount(chain = asset.id.chain).address,
)
