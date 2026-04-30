package com.gemwallet.android.data.service.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Embedded
import com.gemwallet.android.model.TransactionExtended
import com.wallet.core.primitives.Price

data class DbTransactionExtended(
    @Embedded val transaction: DbTransaction,
    @Embedded(prefix = "asset_") val asset: DbAssetProjection,
    @Embedded(prefix = "fee_asset_") val feeAsset: DbAssetProjection,
    @ColumnInfo("price_value") val priceValue: Double?,
    @ColumnInfo("price_day_changed") val priceDayChanged: Double?,
    @ColumnInfo("fee_price_value") val feePriceValue: Double?,
    @ColumnInfo("fee_price_day_changed") val feePriceDayChanged: Double?,
    @Embedded(prefix = "from_asset_") val fromAsset: DbAssetProjection?,
    @Embedded(prefix = "to_asset_") val toAsset: DbAssetProjection?,
)

fun DbTransactionExtended.toDTO(): TransactionExtended? {
    val tx = try {
        transaction.toDTO()
    } catch (_: IllegalArgumentException) {
        return null
    }
    return TransactionExtended(
        transaction = tx,
        asset = asset.toDTO() ?: return null,
        feeAsset = feeAsset.toDTO() ?: return null,
        price = priceValue?.let { Price(it, priceDayChanged ?: 0.0, 0L) },
        feePrice = feePriceValue?.let { Price(it, feePriceDayChanged ?: 0.0, 0L) },
        assets = listOfNotNull(
            fromAsset?.toDTO(),
            toAsset?.toDTO(),
        ),
    )
}

fun List<DbTransactionExtended>.toDTO() = mapNotNull { it.toDTO() }
