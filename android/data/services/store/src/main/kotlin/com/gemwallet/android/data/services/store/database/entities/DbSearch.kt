package com.gemwallet.android.data.services.store.database.entities

import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey

@Entity(
    tableName = "search",
    foreignKeys = [
        ForeignKey(entity = DbAsset::class, parentColumns = ["id"], childColumns = ["assetId"], onDelete = ForeignKey.CASCADE),
        ForeignKey(entity = DbPerpetual::class, parentColumns = ["id"], childColumns = ["perpetualId"], onDelete = ForeignKey.CASCADE),
        ForeignKey(entity = DbAssetList::class, parentColumns = ["id"], childColumns = ["listId"], onDelete = ForeignKey.CASCADE),
    ],
    indices = [
        Index(value = ["query"]),
        Index(value = ["assetId", "query"], unique = true),
        Index(value = ["perpetualId", "query"], unique = true),
        Index(value = ["listId", "query"], unique = true),
    ],
)
data class DbSearch(@PrimaryKey(autoGenerate = true) val id: Long = 0, val query: String, val assetId: String? = null, val perpetualId: String? = null, val listId: String? = null, val priority: Int)

fun List<String>.toAssetSearchRecords(query: String): List<DbSearch> = mapIndexed { index, id -> DbSearch(query = query, assetId = id, priority = index) }

fun List<String>.toPerpetualSearchRecords(query: String): List<DbSearch> = mapIndexed { index, id -> DbSearch(query = query, perpetualId = id, priority = index) }

fun List<String>.toListSearchRecords(query: String): List<DbSearch> = mapIndexed { index, id -> DbSearch(query = query, listId = id, priority = index) }
