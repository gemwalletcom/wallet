package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy.Companion.REPLACE
import androidx.room.Query
import androidx.room.Transaction
import com.gemwallet.android.data.service.store.database.entities.DbAssetPriority
import kotlinx.coroutines.flow.Flow

@Dao
interface AssetsPriorityDao {

    @Insert(onConflict = REPLACE)
    suspend fun insert(priorities: List<DbAssetPriority>)

    @Query("DELETE FROM assets_priority WHERE `query` = :query")
    suspend fun deleteByQuery(query: String)

    @Transaction
    suspend fun put(priorities: List<DbAssetPriority>) {
        priorities.firstOrNull()?.query?.let { deleteByQuery(it) }
        insert(priorities)
    }

    @Query("""
        SELECT COUNT(asset_id) FROM assets_priority WHERE `query` = :query
    """)
    fun hasPriorities(query: String): Flow<Int>
}
