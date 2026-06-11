package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy.Companion.REPLACE
import androidx.room.Query
import androidx.room.Transaction
import com.gemwallet.android.data.service.store.database.entities.DbSearchPriority
import kotlinx.coroutines.flow.Flow

@Dao
interface SearchPriorityDao {

    @Insert(onConflict = REPLACE)
    suspend fun insert(priorities: List<DbSearchPriority>)

    @Query("DELETE FROM search WHERE `query` = :query AND type = :type")
    suspend fun deleteByQuery(query: String, type: String)

    @Transaction
    suspend fun put(priorities: List<DbSearchPriority>) {
        priorities.firstOrNull()?.let { deleteByQuery(it.query, it.type) }
        insert(priorities)
    }

    @Query("SELECT COUNT(item_id) FROM search WHERE `query` = :query AND type = :type")
    fun hasPriorities(query: String, type: String): Flow<Int>
}
