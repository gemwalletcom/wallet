package com.gemwallet.android.data.service.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.PrimaryKey
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetFull
import com.wallet.core.primitives.AssetPrice
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatRate

@Entity(tableName = "prices")
data class DbPrice(
    @PrimaryKey @ColumnInfo("asset_id") val assetId: String,
    val value: Double? = 0.0,
    @ColumnInfo("usd_value") val usdValue: Double? = 0.0,
    @ColumnInfo("day_changed") val dayChanged: Double? = 0.0,
    val currency: Currency,
    val updatedAt: Long? = null,
)

fun AssetPrice.toRecord(rate: FiatRate): DbPrice {
    return DbPrice(
        assetId = assetId.toIdentifier(),
        value = price * rate.rate,
        usdValue = price,
        dayChanged = priceChangePercentage24h,
        currency = rate.symbol,
        updatedAt = updatedAt,
    )
}

fun List<AssetPrice>.toRecord(rate: FiatRate) = map { it.toRecord(rate) }

fun AssetFull.toPriceRecord(rate: FiatRate): DbPrice? {
    return price?.let { price ->
        DbPrice(
            assetId = asset.id.toIdentifier(),
            value = price.price * rate.rate,
            usdValue = price.price,
            dayChanged = price.priceChangePercentage24h,
            currency = rate.symbol,
            updatedAt = price.updatedAt,
        )
    }
}

fun AssetBasic.toPriceRecord(rate: FiatRate): DbPrice? {
    return price?.let { price ->
        DbPrice(
            assetId = asset.id.toIdentifier(),
            value = price.price * rate.rate,
            usdValue = price.price,
            dayChanged = price.priceChangePercentage24h,
            currency = rate.symbol,
            updatedAt = price.updatedAt,
        )
    }
}

fun List<AssetBasic>.toPriceRecord(rate: FiatRate) = mapNotNull { it.toPriceRecord(rate) }
