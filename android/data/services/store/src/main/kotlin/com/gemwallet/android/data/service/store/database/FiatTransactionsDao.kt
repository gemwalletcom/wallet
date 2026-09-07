package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Transaction
import com.gemwallet.android.data.service.store.database.entities.DbFiatTransaction
import com.gemwallet.android.data.service.store.database.entities.DbFiatTransactionWithAsset
import kotlinx.coroutines.flow.Flow

@Dao
interface FiatTransactionsDao {

    @Insert(entity = DbFiatTransaction::class, onConflict = OnConflictStrategy.REPLACE)
    suspend fun insert(transactions: List<DbFiatTransaction>)

    @Query("DELETE FROM fiat_transactions WHERE walletId = :walletId AND id NOT IN (:ids)")
    suspend fun deleteExcept(walletId: String, ids: List<String>)

    @Transaction
    suspend fun setFiatTransactions(walletId: String, transactions: List<DbFiatTransaction>) {
        deleteExcept(walletId, transactions.map { it.id })
        insert(transactions)
    }

    @Transaction
    @Query("SELECT * FROM fiat_transactions WHERE walletId = :walletId ORDER BY createdAt DESC")
    fun getFiatTransactions(walletId: String): Flow<List<DbFiatTransactionWithAsset>>
}
