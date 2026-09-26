package com.gemwallet.android.data.services.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import com.wallet.core.primitives.CoreListItem
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
    @ColumnInfo("wallet_id") val walletId: WalletId,
    @ColumnInfo("read_at") val readAt: Long?,
    @ColumnInfo("created_at") val createdAt: Long,
    @ColumnInfo("item") val item: CoreListItem,
)
