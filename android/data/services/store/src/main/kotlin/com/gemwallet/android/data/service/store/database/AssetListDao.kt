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
        JOIN search ON asset_lists.id = search.listId
        WHERE search.`query` = :query
        ORDER BY search.priority ASC
    """)
    fun searchWithPriority(query: String): Flow<List<DbAssetList>>
}
