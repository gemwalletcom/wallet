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
    suspend fun insert(priorities: List<DbSearch>)

    @Query("DELETE FROM search WHERE `query` = :query AND type = :type")
    suspend fun deleteByQuery(query: String, type: String)

    @Transaction
    suspend fun put(priorities: List<DbSearch>) {
        priorities.firstOrNull()?.let { deleteByQuery(it.query, it.type) }
        insert(priorities)
    }

    @Query("SELECT COUNT(item_id) FROM search WHERE `query` = :query AND type = :type")
    fun hasPriorities(query: String, type: String): Flow<Int>
}
