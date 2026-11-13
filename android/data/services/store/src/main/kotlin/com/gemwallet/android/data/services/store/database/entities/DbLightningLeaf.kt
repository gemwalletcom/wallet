package com.gemwallet.android.data.services.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.ForeignKey

@Entity(
    tableName = "lightning_leaves",
    primaryKeys = ["wallet_id", "id"],
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
data class DbLightningLeaf(@ColumnInfo(name = "wallet_id") val walletId: String, val id: String, val data: String)
