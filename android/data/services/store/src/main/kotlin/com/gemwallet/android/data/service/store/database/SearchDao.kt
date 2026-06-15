package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy.Companion.REPLACE
import androidx.room.Query
import androidx.room.Transaction
import com.gemwallet.android.data.service.store.database.entities.DbSearch
import kotlinx.coroutines.flow.Flow

@Dao
interface SearchDao {

    @Insert(onConflict = REPLACE)
    suspend fun insert(records: List<DbSearch>)

    @Query("DELETE FROM search WHERE `query` = :query AND assetId IS NOT NULL")
    suspend fun deleteAssets(query: String)

    @Query("DELETE FROM search WHERE `query` = :query AND perpetualId IS NOT NULL")
    suspend fun deletePerpetuals(query: String)

    @Transaction
    suspend fun put(records: List<DbSearch>) {
        val first = records.firstOrNull() ?: return
        if (first.assetId != null) deleteAssets(first.query) else deletePerpetuals(first.query)
        insert(records)
    }

    @Query("SELECT COUNT(*) FROM search WHERE `query` = :query AND assetId IS NOT NULL")
    fun hasAssetPriorities(query: String): Flow<Int>

    @Query("SELECT COUNT(*) FROM search WHERE `query` = :query AND perpetualId IS NOT NULL")
    fun hasPerpetualPriorities(query: String): Flow<Int>
}
