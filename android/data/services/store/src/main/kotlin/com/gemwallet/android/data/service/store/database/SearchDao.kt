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

    @Query("DELETE FROM search WHERE `query` = :query AND listId IS NOT NULL")
    suspend fun deleteLists(query: String)

    @Transaction
    suspend fun putAssets(query: String, records: List<DbSearch>) {
        deleteAssets(query)
        insert(records)
    }

    @Transaction
    suspend fun putPerpetuals(query: String, records: List<DbSearch>) {
        deletePerpetuals(query)
        insert(records)
    }

    @Transaction
    suspend fun putLists(query: String, records: List<DbSearch>) {
        deleteLists(query)
        insert(records)
    }

    @Query("SELECT COUNT(*) FROM search WHERE `query` = :query AND assetId IS NOT NULL")
    fun hasAssetPriorities(query: String): Flow<Int>

    @Query("SELECT COUNT(*) FROM search WHERE `query` = :query AND perpetualId IS NOT NULL")
    fun hasPerpetualPriorities(query: String): Flow<Int>
}
