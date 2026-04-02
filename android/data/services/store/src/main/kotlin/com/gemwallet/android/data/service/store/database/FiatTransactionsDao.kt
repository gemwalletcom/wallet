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
    fun insert(transactions: List<DbFiatTransaction>)

    @Transaction
    @Query("SELECT * FROM fiat_transactions WHERE walletId = :walletId ORDER BY createdAt DESC")
    fun getFiatTransactions(walletId: String): Flow<List<DbFiatTransactionWithAsset>>
}
