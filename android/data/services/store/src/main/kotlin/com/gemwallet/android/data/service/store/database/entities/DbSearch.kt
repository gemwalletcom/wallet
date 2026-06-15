package com.gemwallet.android.data.service.store.database.entities

import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.PerpetualSearchData

@Entity(
    tableName = "search",
    foreignKeys = [
        ForeignKey(entity = DbAsset::class, parentColumns = ["id"], childColumns = ["assetId"], onDelete = ForeignKey.CASCADE),
        ForeignKey(entity = DbPerpetual::class, parentColumns = ["id"], childColumns = ["perpetualId"], onDelete = ForeignKey.CASCADE),
    ],
    indices = [
        Index(value = ["query"]),
        Index(value = ["assetId", "query"], unique = true),
        Index(value = ["perpetualId", "query"], unique = true),
    ],
)
data class DbSearch(
    @PrimaryKey(autoGenerate = true) val id: Long = 0,
    val query: String,
    val assetId: String? = null,
    val perpetualId: String? = null,
    val priority: Int,
)

@JvmName("assetsToSearchRecord")
fun List<AssetBasic>.toSearchRecord(query: String): List<DbSearch> = mapIndexed { index, basic ->
    DbSearch(query = query, assetId = basic.asset.id.toIdentifier(), priority = index)
}

@JvmName("perpetualsToSearchRecord")
fun List<PerpetualSearchData>.toSearchRecord(query: String): List<DbSearch> = mapIndexed { index, data ->
    DbSearch(query = query, perpetualId = data.perpetual.id.toIdentifier(), priority = index)
}
