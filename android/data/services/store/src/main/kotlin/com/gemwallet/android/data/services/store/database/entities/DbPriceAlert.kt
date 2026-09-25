package com.gemwallet.android.data.services.store.database.entities

import androidx.room.Embedded
import androidx.room.Entity
import androidx.room.PrimaryKey
import androidx.room.Relation
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.PriceAlertInfo
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Price
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertData
import com.wallet.core.primitives.PriceAlertDirection
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

@Entity(tableName = "price_alerts")
data class DbPriceAlert(
    @PrimaryKey val id: String,
    val assetId: String,
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
        priceAlert = alert.toDTO().priceAlert,
        rankScore = asset.rank,
    )
}

private fun DbPrice.toPrice(): Price? = value?.takeIf { it > 0 }?.let {
    Price(price = it, priceChangePercentage24h = dayChanged ?: 0.0, updatedAt = updatedAt ?: 0)
}

fun DbPriceAlert.toDTO(): PriceAlertInfo = PriceAlertInfo(
    priceAlert = PriceAlert(
        assetId = assetId.toAssetId() ?: throw IllegalStateException(),
        price = price,
        priceDirection = priceDirection,
        pricePercentChange = pricePercentChange,
        currency = currency,
        lastNotifiedAt = lastNotifiedAt,
    ),
)

fun PriceAlert.toRecord(id: String): DbPriceAlert = DbPriceAlert(
    id = id,
    assetId = assetId.toIdentifier(),
    price = price,
    pricePercentChange = pricePercentChange,
    priceDirection = priceDirection,
    currency = currency,
    lastNotifiedAt = lastNotifiedAt,
)

fun List<DbPriceAlert>.toDTO() = map { it.toDTO() }

fun Flow<DbPriceAlert>.toDTO() = map { it.toDTO() }
