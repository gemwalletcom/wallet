package com.gemwallet.android.data.service.store.database.entities

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
import com.wallet.core.primitives.PerpetualPosition
import com.wallet.core.primitives.PerpetualPositionData
import com.wallet.core.primitives.PerpetualTriggerOrder

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
    val updatedAt: Long,
)

data class DbPerpetualPositionData(
    @Embedded
    val position: DbPerpetualPosition,

    @Relation(parentColumn = "perpetualId", entityColumn = "id")
    val perpetual: DbPerpetual,

    @Relation(parentColumn = "assetId", entityColumn = "id")
    val asset: DbAsset,
)

fun DbPerpetualPosition.toDto(): PerpetualPosition {
    val takeProfitTrigger = if (takeProfitType != null && takeProfitPrice != null && takeProfitOrderId != null) {
        PerpetualTriggerOrder(
            price = takeProfitPrice,
            order_type = takeProfitType,
            order_id = takeProfitOrderId,
        )
    } else null

    val stopLossTrigger = if (stopLossType != null && stopLossPrice != null && stopLossOrderId != null) {
        PerpetualTriggerOrder(
            price = stopLossPrice,
            order_type = stopLossType,
            order_id = stopLossOrderId,
        )
    } else null

    return PerpetualPosition(
        id = id,
        perpetualId = perpetualId,
        assetId = assetId,
        size = size,
        sizeValue = sizeValue,
        leverage = leverage.toUByte(),
        entryPrice = entryPrice ?: 0.0,
        liquidationPrice = liquidationPrice,
        marginType = marginType,
        direction = direction,
        marginAmount = marginAmount,
        takeProfit = takeProfitTrigger,
        stopLoss = stopLossTrigger,
        pnl = pnl,
        funding = funding,
    )
}

fun PerpetualPosition.toDB(walletId: String, updatedAt: Long = System.currentTimeMillis()): DbPerpetualPosition = DbPerpetualPosition(
    id = id,
    walletId = walletId,
    perpetualId = perpetualId,
    assetId = assetId,
    size = size,
    sizeValue = sizeValue,
    leverage = leverage.toInt(),
    entryPrice = entryPrice,
    liquidationPrice = liquidationPrice,
    marginType = marginType,
    direction = direction,
    marginAmount = marginAmount,
    takeProfitPrice = takeProfit?.price,
    takeProfitType = takeProfit?.order_type,
    takeProfitOrderId = takeProfit?.order_id,
    stopLossPrice = stopLoss?.price,
    stopLossType = stopLoss?.order_type,
    stopLossOrderId = stopLoss?.order_id,
    pnl = pnl,
    funding = funding,
    updatedAt = updatedAt,
)

fun DbPerpetualPositionData.toDTO(): PerpetualPositionData? {
    val asset = asset.toDTO() ?: return null
    return PerpetualPositionData(
        perpetual = perpetual.toDTO(),
        asset = asset,
        position = position.toDto(),
    )
}
