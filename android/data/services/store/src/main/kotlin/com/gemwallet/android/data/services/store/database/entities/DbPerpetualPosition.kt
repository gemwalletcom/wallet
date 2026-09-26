package com.gemwallet.android.data.services.store.database.entities

import androidx.room.Embedded
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.Relation
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualOrderType
import com.wallet.core.primitives.PerpetualPositionData

@Entity(
    tableName = "perpetuals_positions",
    primaryKeys = ["id", "walletId"],
    foreignKeys = [
        ForeignKey(
            entity = DbWallet::class,
            parentColumns = ["id"],
            childColumns = ["walletId"],
            onDelete = ForeignKey.CASCADE,
            onUpdate = ForeignKey.CASCADE,
        ),
        ForeignKey(
            entity = DbPerpetual::class,
            parentColumns = ["id"],
            childColumns = ["perpetualId"],
            onDelete = ForeignKey.CASCADE,
            onUpdate = ForeignKey.CASCADE,
        ),
        ForeignKey(
            entity = DbAsset::class,
            parentColumns = ["id"],
            childColumns = ["assetId"],
            onDelete = ForeignKey.CASCADE,
            onUpdate = ForeignKey.CASCADE,
        ),
    ],
    indices = [
        Index(name = "perpetuals_positions_wallet_id_idx", value = ["walletId"]),
        Index(name = "perpetuals_positions_perpetual_id_idx", value = ["perpetualId"]),
        Index(name = "perpetuals_positions_asset_id_idx", value = ["assetId"]),
    ],
)
data class DbPerpetualPosition(
    val id: String,
    val walletId: String,
    val perpetualId: PerpetualId,
    val assetId: AssetId,
    val size: Double,
    val sizeValue: Double,
    val leverage: Int,
    val entryPrice: Double? = null,
    val liquidationPrice: Double? = null,
    val marginType: PerpetualMarginType,
    val direction: PerpetualDirection,
    val marginAmount: Double,
    val takeProfitPrice: Double? = null,
    val takeProfitType: PerpetualOrderType? = null,
    val takeProfitOrderId: String? = null,
    val stopLossPrice: Double? = null,
    val stopLossType: PerpetualOrderType? = null,
    val stopLossOrderId: String? = null,
    val pnl: Double,
    val funding: Float? = null,
    val updatedAt: Long = System.currentTimeMillis(),
)

data class DbPerpetualPositionData(
    @Embedded
    val position: DbPerpetualPosition,

    @Relation(parentColumn = "perpetualId", entityColumn = "id")
    val perpetual: DbPerpetual,

    @Relation(parentColumn = "assetId", entityColumn = "id")
    val asset: DbAsset,
)

fun DbPerpetualPositionData.toDTO(): PerpetualPositionData? {
    val asset = asset.toDTO() ?: return null
    return PerpetualPositionData(
        perpetual = perpetual.toPerpetual(),
        asset = asset,
        position = position.toPerpetualPosition(),
    )
}
