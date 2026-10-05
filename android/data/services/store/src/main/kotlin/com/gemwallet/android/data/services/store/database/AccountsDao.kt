package com.gemwallet.android.data.services.store.database

import androidx.room.Dao
import androidx.room.Delete
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Transaction
import com.gemwallet.android.data.services.store.database.entities.DbAccount
import com.gemwallet.android.data.services.store.database.entities.DbWalletAddress
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.flow.Flow

@Dao
interface AccountsDao {
    @Query("SELECT * FROM accounts WHERE wallet_id = :walletId")
    suspend fun getByWalletId(walletId: String): List<DbAccount>

    @Transaction
    @Query("SELECT * FROM accounts WHERE chain = :chain")
    fun getWalletAddresses(chain: Chain): Flow<List<DbWalletAddress>>

    @Query("SELECT * FROM accounts WHERE chain = :chain AND address = :address")
    suspend fun getByAddress(chain: Chain, address: String): List<DbAccount>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insert(account: DbAccount)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insert(account: List<DbAccount>)

    @Delete
    suspend fun delete(account: DbAccount)

    @Query("DELETE FROM accounts WHERE wallet_id=:walletId")
    suspend fun deleteByWalletId(walletId: String)
}
