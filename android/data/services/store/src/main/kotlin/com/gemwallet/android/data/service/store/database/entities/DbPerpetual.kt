package com.gemwallet.android.data.service.store.database.entities

import androidx.room.Embedded
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey
import androidx.room.Relation
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Perpetual
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualMetadata
import com.wallet.core.primitives.PerpetualProvider

@Entity(
    tableName = "perpetuals",
    foreignKeys = [
        ForeignKey(
            entity = DbAsset::class,
            parentColumns = ["id"],
            childColumns = ["assetId"],
            onDelete = ForeignKey.CASCADE,
            onUpdate = ForeignKey.CASCADE,
        ),
    ],
    indices = [Index(name = "perpetuals_asset_id_idx", value = ["assetId"])],
)
data class DbPerpetual(
    @PrimaryKey val id: String,
    val name: String,
    val provider: PerpetualProvider,
    val assetId: String,
    val identifier: String,
    val price: Double,
    val pricePercentChange24h: Double,
    val openInterest: Double,
    val volume24h: Double,
    val funding: Double,
    val maxLeverage: Int,
    val isIsolatedOnly: Boolean = false,
    val isPinned: Boolean = false,
)

data class DbPerpetualUpdate(
    val id: String,
    val name: String,
    val provider: PerpetualProvider,
    val assetId: String,
    val identifier: String,
    val price: Double,
    val pricePercentChange24h: Double,
    val openInterest: Double,
    val volume24h: Double,
    val funding: Double,
    val maxLeverage: Int,
    val isIsolatedOnly: Boolean,
)

fun DbPerpetual.toUpdate() = DbPerpetualUpdate(
    id = id,
    name = name,
    provider = provider,
    assetId = assetId,
    identifier = identifier,
    price = price,
    pricePercentChange24h = pricePercentChange24h,
    openInterest = openInterest,
    volume24h = volume24h,
    funding = funding,
    maxLeverage = maxLeverage,
    isIsolatedOnly = isIsolatedOnly,
)

data class DbPerpetualData(
    @Embedded
    val perpetual: DbPerpetual,

    @Relation(parentColumn = "assetId", entityColumn = "id")
    val asset: DbAsset,
)

fun DbPerpetual.toDTO(): Perpetual? {
    return Perpetual(
        id = id,
        name = name,
        provider = provider,
        assetId = assetId.toAssetId() ?: return null,
        identifier = identifier,
        price = price,
        pricePercentChange24h = pricePercentChange24h,
        openInterest = openInterest,
        volume24h = volume24h,
        funding = funding,
        maxLeverage = maxLeverage.toUByte(),
        isIsolatedOnly = isIsolatedOnly,
    )
}

fun Perpetual.toDB(isPinned: Boolean = false): DbPerpetual {
    return DbPerpetual(
        id = id,
        name = name,
        provider = provider,
        assetId = assetId.toIdentifier(),
        identifier = identifier,
        price = price,
        pricePercentChange24h = pricePercentChange24h,
        openInterest = openInterest,
        volume24h = volume24h,
        funding = funding,
        maxLeverage = maxLeverage.toInt(),
        isIsolatedOnly = isIsolatedOnly,
        isPinned = isPinned,
    )
}

fun DbPerpetualData.toDTO(): PerpetualData? {
    return PerpetualData(
        perpetual = perpetual.toDTO() ?: return null,
        asset = asset.toDTO() ?: return null,
        metadata = PerpetualMetadata(perpetual.isPinned),
    )
}
