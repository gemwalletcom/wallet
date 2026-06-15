package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Transaction
import com.gemwallet.android.data.service.store.database.entities.DbSupportMessage
import kotlinx.coroutines.flow.Flow

@Dao
interface SupportMessagesDao {

    @Query("SELECT * FROM support_messages ORDER BY createdAt ASC, id ASC")
    fun getMessages(): Flow<List<DbSupportMessage>>

    @Query("UPDATE support_messages SET status = :failed WHERE status = :sending")
    suspend fun failPending(sending: String, failed: String)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun addMessages(messages: List<DbSupportMessage>)

    @Query("DELETE FROM support_messages WHERE id = :id")
    suspend fun delete(id: String)

    @Transaction
    suspend fun replace(oldId: String, message: DbSupportMessage) {
        delete(oldId)
        addMessages(listOf(message))
    }
}
