package com.gemwallet.android.data.service.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Entity
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.PerpetualSearchData
import com.wallet.core.primitives.SearchItemType

@Entity(
    tableName = "search",
    primaryKeys = ["query", "type", "item_id"],
)
data class DbSearchPriority(
    val query: String,
    val type: String,
    @ColumnInfo(name = "item_id") val itemId: String,
    val priority: Int,
)

@JvmName("assetsToSearchPriority")
fun List<AssetBasic>.toSearchPriority(query: String): List<DbSearchPriority> = mapIndexed { index, basic ->
    DbSearchPriority(
        query = query,
        type = SearchItemType.Asset.string,
        itemId = basic.asset.id.toIdentifier(),
        priority = index,
    )
}

@JvmName("perpetualsToSearchPriority")
fun List<PerpetualSearchData>.toSearchPriority(query: String): List<DbSearchPriority> = mapIndexed { index, data ->
    DbSearchPriority(
        query = query,
        type = SearchItemType.Perpetual.string,
        itemId = data.perpetual.id.toIdentifier(),
        priority = index,
    )
}
