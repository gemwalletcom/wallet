package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Delete
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import com.gemwallet.android.data.service.store.database.entities.DbAccount
import com.gemwallet.android.data.service.store.database.entities.DbWallet
import kotlinx.coroutines.flow.Flow

@Dao
interface WalletsDao {
    @Query("""
        SELECT * FROM wallets
        LEFT JOIN accounts ON wallets.id = accounts.wallet_id
    """)
    fun getAll(): Flow<Map<DbWallet, List<DbAccount>>>

    @Query("""
        SELECT * FROM wallets
        LEFT JOIN accounts ON wallets.id = accounts.wallet_id
    """)
    fun getAllNow(): Map<DbWallet, List<DbAccount>>

    @Query("SELECT * FROM wallets WHERE id = :id")
    fun getById(id: String): Flow<DbWallet?>

    @Query("""
        SELECT * FROM wallets
        LEFT JOIN accounts ON wallets.id = accounts.wallet_id
        WHERE wallets.id = :id
    """)
    fun getByIdNow(id: String): Map<DbWallet, List<DbAccount>>

    @Insert(onConflict = OnConflictStrategy.IGNORE)
    suspend fun insert(wallet: DbWallet)

    @Query("UPDATE wallets SET pinned = :pinned WHERE id = :id")
    suspend fun setPinned(id: String, pinned: Boolean)

    @Query("UPDATE wallets SET name = :name WHERE id = :id")
    suspend fun setName(id: String, name: String)

    @Query("UPDATE wallets SET imageUrl = :imageUrl WHERE id = :id")
    suspend fun setImageUrl(id: String, imageUrl: String?)

    @Delete
    suspend fun delete(account: DbWallet)
}