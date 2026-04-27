package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.RawQuery
import androidx.sqlite.db.SupportSQLiteQuery
import com.gemwallet.android.application.transactions.coordinators.TransactionsRequestFilter
import com.gemwallet.android.data.service.store.database.entities.DbAccount
import com.gemwallet.android.data.service.store.database.entities.DbAsset
import com.gemwallet.android.data.service.store.database.entities.DbPrice
import com.gemwallet.android.data.service.store.database.entities.DbSession
import com.gemwallet.android.data.service.store.database.entities.DbTransaction
import com.gemwallet.android.data.service.store.database.entities.DbTransactionExtended
import com.gemwallet.android.data.service.store.database.entities.DbTxSwapMetadata
import com.wallet.core.primitives.TransactionState
import kotlinx.coroutines.flow.Flow

@Dao
interface TransactionsDao {

    @Insert(entity = DbTransaction::class, onConflict = OnConflictStrategy.REPLACE)
    fun insert(transactions: List<DbTransaction>)

    @Query("DELETE FROM transactions WHERE id=:id")
    fun delete(id: String)

    @Query("SELECT * FROM transactions WHERE state = :state")
    fun getByState(state: TransactionState): List<DbTransaction>

    @RawQuery(
        observedEntities = [
            DbTransaction::class,
            DbAsset::class,
            DbPrice::class,
            DbTxSwapMetadata::class,
            DbAccount::class,
            DbSession::class,
        ]
    )
    fun getExtendedTransactions(query: SupportSQLiteQuery): Flow<List<DbTransactionExtended>>

    fun getExtendedTransactions(
        filters: List<TransactionsRequestFilter> = emptyList(),
    ): Flow<List<DbTransactionExtended>> = getExtendedTransactions(buildExtendedTransactionsSql(filters).toSupportSQLiteQuery())

    @Query("SELECT * FROM extended_txs WHERE state = :state ORDER BY createdAt DESC")
    fun getExtendedTransactions(state: TransactionState): Flow<List<DbTransactionExtended>>

    @Query("SELECT COUNT(*) FROM extended_txs WHERE state = 'Pending' ORDER BY createdAt DESC")
    fun getPendingCount(): Flow<Int?>

    @Query("SELECT * FROM extended_txs WHERE id = :id")
    fun getExtendedTransaction(id: String): Flow<DbTransactionExtended?>

    @Query("SELECT * FROM extended_txs WHERE state = :state ORDER BY createdAt DESC")
    fun getTransactionsByState(state: TransactionState): Flow<List<DbTransactionExtended>>

    @Insert(entity = DbTxSwapMetadata::class, onConflict = OnConflictStrategy.REPLACE)
    fun addSwapMetadata(metadata: List<DbTxSwapMetadata>)

    @Query("SELECT * FROM tx_swap_metadata WHERE tx_id=:txId")
    fun getMetadata(txId: String): DbTxSwapMetadata?

    @Query("DELETE FROM transactions WHERE state = 'Pending'")
    fun removePendingTransactions()
}
