package com.gemwallet.android.data.services.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.ForeignKey

@Entity(
    tableName = "lightning_pending_sends",
    primaryKeys = ["wallet_id", "transfer_id"],
    foreignKeys = [
        ForeignKey(
            entity = DbWallet::class,
            parentColumns = ["id"],
            childColumns = ["wallet_id"],
            onDelete = ForeignKey.CASCADE,
            onUpdate = ForeignKey.CASCADE,
        ),
    ],
)
data class DbLightningPendingSend(
    @ColumnInfo(name = "wallet_id") val walletId: String,
    @ColumnInfo(name = "transfer_id") val transferId: String,
    val invoice: String,
    @ColumnInfo(name = "amount_sats") val amountSats: Long?,
    @ColumnInfo(name = "expires_at") val expiresAt: Long,
)
