package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Query
import androidx.room.Upsert
import com.gemwallet.android.data.service.store.database.entities.DbAssetList
import kotlinx.coroutines.flow.Flow

@Dao
interface AssetListDao {

    @Upsert
    suspend fun upsert(items: List<DbAssetList>)

    @Query("""
        SELECT asset_lists.* FROM asset_lists
        JOIN search ON search.listId = asset_lists.id AND search.`query` = :query
        ORDER BY search.priority ASC
    """)
    fun search(query: String): Flow<List<DbAssetList>>
}
