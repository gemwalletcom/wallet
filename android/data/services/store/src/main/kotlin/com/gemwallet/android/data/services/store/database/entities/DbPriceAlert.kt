package com.gemwallet.android.data.services.store.database.entities

import androidx.room.Embedded
import androidx.room.Entity
import androidx.room.PrimaryKey
import androidx.room.Relation
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlertData
import com.wallet.core.primitives.PriceAlertDirection

@Entity(tableName = "price_alerts")
data class DbPriceAlert(
    @PrimaryKey val id: String,
    val assetId: AssetId,
    val currency: Currency,
    val price: Double? = null,
    val pricePercentChange: Double? = null,
    val priceDirection: PriceAlertDirection? = null,
    val lastNotifiedAt: Long? = null,
)

data class DbPriceAlertWithAsset(@Embedded val alert: DbPriceAlert, @Relation(parentColumn = "assetId", entityColumn = "id") val asset: DbAsset, @Relation(parentColumn = "assetId", entityColumn = "asset_id") val price: DbPrice?)

fun DbPriceAlertWithAsset.toDTO(): PriceAlertData? = asset.toDTO()?.let {
    PriceAlertData(
        asset = it,
        price = price?.toPrice(),
        priceAlert = alert.toPriceAlert(),
        rankScore = asset.rank,
    )
}
