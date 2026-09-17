package com.gemwallet.android.data.service.store.database.entities

import com.gemwallet.android.testkit.mockAsset
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualProvider

fun mockDbPerpetualData(
    asset: Asset = mockAsset(),
    identifier: String = "BTC-PERP",
) = DbPerpetualData(
    perpetual = DbPerpetual(
        id = PerpetualId(PerpetualProvider.Hypercore, identifier),
        name = "${asset.name} Perpetual",
        provider = PerpetualProvider.Hypercore,
        assetId = asset.id,
        identifier = identifier,
        price = 1.0,
        pricePercentChange24h = 0.0,
        openInterest = 0.0,
        volume24h = 0.0,
        funding = 0.0,
        maxLeverage = 1,
    ),
    asset = asset.toRecord(),
)
