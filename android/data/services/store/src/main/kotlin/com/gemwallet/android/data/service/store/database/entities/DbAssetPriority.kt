package com.gemwallet.android.data.service.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Entity
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetBasic

@Entity(
    tableName = "assets_priority",
    primaryKeys = ["query", "asset_id"],
)
data class DbAssetPriority(
    val query: String,
    @ColumnInfo(name = "asset_id") val assetId: String,
    val priority: Int,
)

fun List<AssetBasic>.toRecordPriority(query: String): List<DbAssetPriority> = mapIndexed { index, basic ->
    DbAssetPriority(
        query = query,
        assetId = basic.asset.id.toIdentifier(),
        priority = index,
    )
}
