package com.gemwallet.android.data.services.store.database.entities

import androidx.room.Embedded
import androidx.room.Relation
import com.wallet.core.primitives.PriceData

data class DbPriceInfo(
    @Embedded val asset: DbAsset,
    @Relation(parentColumn = "id", entityColumn = "asset_id") val price: DbPrice?,
    @Relation(parentColumn = "id", entityColumn = "asset_id") val market: DbAssetMarket?,
    @Relation(parentColumn = "id", entityColumn = "asset_id") val links: List<DbAssetLink>,
    @Relation(parentColumn = "id", entityColumn = "assetId") val priceAlerts: List<DbPriceAlert>,
)

fun DbPriceInfo.toDTO(): PriceData? = asset.toDTO()?.let {
    PriceData(
        asset = it,
        price = price?.toPrice(),
        priceAlerts = priceAlerts.map { alert -> alert.toDTO().priceAlert },
        market = market?.toDTO(),
        links = links.toAssetLinksModel(),
    )
}
