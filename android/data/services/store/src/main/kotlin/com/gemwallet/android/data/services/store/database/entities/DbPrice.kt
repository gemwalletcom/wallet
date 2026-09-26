package com.gemwallet.android.data.services.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.PrimaryKey
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Price

@Entity(tableName = "prices")
data class DbPrice(
    @PrimaryKey @ColumnInfo("asset_id") val assetId: String,
    val value: Double? = 0.0,
    @ColumnInfo("usd_value") val usdValue: Double? = 0.0,
    @ColumnInfo("day_changed") val dayChanged: Double? = 0.0,
    val currency: Currency,
    val updatedAt: Long? = null,
)

fun DbPrice.toPrice(): Price? = value?.takeIf { it > 0 }?.let {
    Price(price = it, priceChangePercentage24h = dayChanged ?: 0.0, updatedAt = updatedAt ?: 0)
}
