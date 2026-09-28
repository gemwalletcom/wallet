package com.gemwallet.android.data.services.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.Index

@Entity(tableName = "transactions_assets", primaryKeys = ["tx_id", "asset_id"], indices = [Index("asset_id")])
data class DbTransactionAsset(@ColumnInfo(name = "tx_id") val transactionId: String, @ColumnInfo(name = "asset_id") val assetId: String)
