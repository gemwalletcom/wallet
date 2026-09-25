package com.gemwallet.android.data.service.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Embedded
import androidx.room.Junction
import androidx.room.Relation
import com.wallet.core.primitives.TransactionListItem

data class DbTransactionListItem(
    @Embedded val transaction: DbTransaction,
    @Embedded(prefix = "asset_") val asset: DbAssetProjection,
    @Embedded(prefix = "from_address_") val fromAddress: DbAddressProjection?,
    @Embedded(prefix = "to_address_") val toAddress: DbAddressProjection?,
    @ColumnInfo("tx_key") val transactionKey: String,
    @Relation(
        entity = DbAsset::class,
        parentColumn = "tx_key",
        entityColumn = "id",
        associateBy = Junction(DbTransactionAsset::class, parentColumn = "tx_id", entityColumn = "asset_id"),
    )
    val assets: List<DbAssetProjection>,
)

fun DbTransactionListItem.toDTO(): TransactionListItem? = TransactionListItem(
    transaction = transaction.toDTO(),
    asset = asset.toDTO() ?: return null,
    assets = assets.mapNotNull { it.toDTO() },
    fromAddress = fromAddress?.toAddressName(transaction.owner),
    toAddress = toAddress?.toAddressName(transaction.recipient),
)

fun List<DbTransactionListItem>.toDTO() = mapNotNull { it.toDTO() }
