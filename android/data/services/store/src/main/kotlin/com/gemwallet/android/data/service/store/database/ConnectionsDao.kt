package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Delete
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Update
import com.gemwallet.android.data.service.store.database.entities.DbConnection
import kotlinx.coroutines.flow.Flow

@Dao
interface ConnectionsDao {
    @Insert(onConflict = OnConflictStrategy.Companion.REPLACE)
    suspend fun insert(connection: DbConnection)

    @Insert(onConflict = OnConflictStrategy.Companion.REPLACE)
    suspend fun insert(connections: List<DbConnection>)

    @Update
    suspend fun update(connection: DbConnection)

    @Query("SELECT * FROM wallets_connections")
    fun getAll(): Flow<List<DbConnection>>

    @Query("SELECT * FROM wallets_connections WHERE id = :connectionId")
    fun getConnection(connectionId: String): Flow<DbConnection?>

    @Query("SELECT * FROM wallets_connections")
    suspend fun getConnections(): List<DbConnection>

    @Query("SELECT * FROM wallets_connections WHERE id = :sessionId OR session_id = :sessionId")
    suspend fun getBySessionId(sessionId: String): DbConnection?

    @Query("DELETE FROM wallets_connections WHERE id IN (:sessionIds) OR session_id IN (:sessionIds)")
    suspend fun delete(sessionIds: List<String>)

    @Query("DELETE FROM wallets_connections")
    suspend fun deleteAll()

    @Delete
    suspend fun deleteAll(sessions: List<DbConnection>)
}
