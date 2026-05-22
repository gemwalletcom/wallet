package com.gemwallet.android.data.service.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import com.wallet.core.primitives.CoreListItem
import com.wallet.core.primitives.InAppNotification
import com.wallet.core.primitives.WalletId

@Entity(
    tableName = "in_app_notifications",
    primaryKeys = ["id"],
    indices = [Index("wallet_id"), Index("created_at")],
    foreignKeys = [
        ForeignKey(DbWallet::class, ["id"], ["wallet_id"], onDelete = ForeignKey.CASCADE, onUpdate = ForeignKey.CASCADE),
    ],
)
data class DbInAppNotification(
    @ColumnInfo("id") val id: String,
    @ColumnInfo("wallet_id") val walletId: String,
    @ColumnInfo("read_at") val readAt: Long?,
    @ColumnInfo("created_at") val createdAt: Long,
    @ColumnInfo("item") val item: CoreListItem,
)

fun DbInAppNotification.toModel(): InAppNotification = InAppNotification(
    walletId = WalletId(walletId),
    readAt = readAt,
    createdAt = createdAt,
    item = item,
)

fun InAppNotification.toRecord(): DbInAppNotification = DbInAppNotification(
    id = item.id,
    walletId = walletId.id,
    readAt = readAt,
    createdAt = createdAt,
    item = item,
)
