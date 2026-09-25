package com.gemwallet.android.data.services.store.database.entities

import androidx.room.Entity
import androidx.room.PrimaryKey
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.PriceAlertInfo
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlert
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
