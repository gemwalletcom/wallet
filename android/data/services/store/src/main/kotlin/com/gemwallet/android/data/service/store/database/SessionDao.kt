package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import com.gemwallet.android.data.service.store.database.entities.DbSession
import com.wallet.core.primitives.Currency
import kotlinx.coroutines.flow.Flow

@Dao
interface SessionDao {
    @Query("SELECT * FROM session WHERE id = 1")
    fun session(): Flow<DbSession?>

    @Query("SELECT * FROM session WHERE id = 1")
    fun getSession(): DbSession?

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun update(session: DbSession)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    fun updateNow(session: DbSession)

    @Query("DELETE FROM session")
    fun clearNow()

    @Query("UPDATE session SET currency = :currency WHERE id = 1")
    suspend fun setCurrency(currency: Currency)

    @Query("SELECT currency FROM session WHERE id = 1")
    suspend fun getCurrency(): Currency?

    @Query("DELETE FROM session")
    suspend fun clear()
}
