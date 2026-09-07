package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.RawQuery
import androidx.room.Transaction
import androidx.room.Update
import androidx.sqlite.db.SupportSQLiteQuery
import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.data.service.store.database.entities.DbAddress
import com.gemwallet.android.data.service.store.database.entities.DbAsset
import com.gemwallet.android.data.service.store.database.entities.DbPrice
import com.gemwallet.android.data.service.store.database.entities.DbSwapPair
import com.gemwallet.android.data.service.store.database.entities.DbTransaction
import com.gemwallet.android.data.service.store.database.entities.DbTransactionExtended
import com.gemwallet.android.data.service.store.database.entities.DbTransactionSwapMetadata
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.emitAll
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.onEach

const val EXTENDED_COLUMNS = """
    tx.*,
    asset.id AS asset_id,
    asset.name AS asset_name,
    asset.symbol AS asset_symbol,
    asset.decimals AS asset_decimals,
    asset.type AS asset_type,
    feeAsset.id AS fee_asset_id,
    feeAsset.name AS fee_asset_name,
    feeAsset.symbol AS fee_asset_symbol,
    feeAsset.decimals AS fee_asset_decimals,
    feeAsset.type AS fee_asset_type,
    prices.value AS price_value,
    prices.day_changed AS price_day_changed,
    feePrices.value AS fee_price_value,
    feePrices.day_changed AS fee_price_day_changed,
    from_prices.value AS from_price_value,
    from_prices.day_changed AS from_price_day_changed,
    to_prices.value AS to_price_value,
    to_prices.day_changed AS to_price_day_changed,
    from_asset.id AS from_asset_id,
    from_asset.name AS from_asset_name,
    from_asset.symbol AS from_asset_symbol,
    from_asset.decimals AS from_asset_decimals,
    from_asset.type AS from_asset_type,
    to_asset.id AS to_asset_id,
    to_asset.name AS to_asset_name,
    to_asset.symbol AS to_asset_symbol,
    to_asset.decimals AS to_asset_decimals,
    to_asset.type AS to_asset_type,
    from_addr.chain AS from_address_chain,
    from_addr.name AS from_address_name,
    from_addr.type AS from_address_type,
    from_addr.status AS from_address_status,
    to_addr.chain AS to_address_chain,
    to_addr.name AS to_address_name,
    to_addr.type AS to_address_type,
    to_addr.status AS to_address_status
"""

const val EXTENDED_SOURCE = """
    FROM transactions as tx
    INNER JOIN asset ON tx.assetId = asset.id
    INNER JOIN asset as feeAsset ON tx.feeAssetId = feeAsset.id
    LEFT JOIN prices ON tx.assetId = prices.asset_id
    LEFT JOIN prices as feePrices ON tx.feeAssetId = feePrices.asset_id
    LEFT JOIN tx_swap_metadata as swap ON tx.id = swap.tx_id
    LEFT JOIN asset as from_asset ON swap.from_asset_id = from_asset.id
    LEFT JOIN asset as to_asset ON swap.to_asset_id = to_asset.id
    LEFT JOIN prices as from_prices ON swap.from_asset_id = from_prices.asset_id
    LEFT JOIN prices as to_prices ON swap.to_asset_id = to_prices.asset_id
    LEFT JOIN addresses as from_addr ON from_addr.chain = asset.chain AND from_addr.address = tx.owner
    LEFT JOIN addresses as to_addr ON to_addr.chain = asset.chain AND to_addr.address = tx.recipient
    WHERE tx.walletId = :walletId
"""

@Dao
interface TransactionsDao {

    @Transaction
    fun insert(transactions: List<DbTransaction>) {
        transactions.forEach { transaction ->
            val existing = getTransaction(transaction.id, transaction.walletId)
            if (existing == null) {
                insertTransaction(transaction)
            } else {
                updateTransaction(transaction.copy(recordId = existing.recordId))
            }
        }
    }

    @Insert
    fun insertTransaction(transaction: DbTransaction)

    @Update
    fun updateTransaction(transaction: DbTransaction)

    @Query("DELETE FROM transactions WHERE id = :id AND walletId = :walletId")
    fun delete(id: TransactionId, walletId: WalletId)

    @RawQuery(
        observedEntities = [
            DbTransaction::class,
            DbAsset::class,
            DbPrice::class,
            DbTransactionSwapMetadata::class,
            DbAddress::class,
        ]
    )
    fun getExtendedTransactions(query: SupportSQLiteQuery): Flow<List<DbTransactionExtended>>

    fun getExtendedTransactions(
        walletId: WalletId,
        filters: List<TransactionsRequestFilter> = emptyList(),
    ): Flow<List<DbTransactionExtended>> = getExtendedTransactions(buildExtendedTransactionsSql(walletId, filters).toSupportSQLiteQuery())

    @RawQuery(
        observedEntities = [
            DbTransaction::class,
            DbAsset::class,
        ]
    )
    fun getTransactionsCount(query: SupportSQLiteQuery): Flow<Int?>

    fun getTransactionsCount(
        walletId: WalletId,
        filters: List<TransactionsRequestFilter>,
    ): Flow<Int?> = getTransactionsCount(buildTransactionsCountSql(walletId, filters).toSupportSQLiteQuery())

    fun getExtendedTransaction(walletId: WalletId, id: TransactionId): Flow<DbTransactionExtended?> = flow {
        val recordId = getTransactionRecordId(walletId, id).onEach { if (it == null) emit(null) }.filterNotNull().first()
        emitAll(getExtendedTransactionByRecordId(walletId, recordId))
    }

    @Query("SELECT recordId FROM transactions WHERE walletId = :walletId AND id = :id")
    fun getTransactionRecordId(walletId: WalletId, id: TransactionId): Flow<Long?>

    @Query("SELECT $EXTENDED_COLUMNS $EXTENDED_SOURCE AND tx.recordId = :recordId")
    fun getExtendedTransactionByRecordId(walletId: WalletId, recordId: Long): Flow<DbTransactionExtended?>

    @Query("SELECT * FROM transactions WHERE state IN (:states)")
    fun getTransactionsByStates(states: List<TransactionState>): List<DbTransaction>

    @Query("SELECT state FROM transactions WHERE id = :id AND walletId = :walletId")
    fun getTransactionState(id: TransactionId, walletId: WalletId): TransactionState?

    @Query("SELECT * FROM transactions WHERE id = :id AND walletId = :walletId")
    fun getTransaction(id: TransactionId, walletId: WalletId): DbTransaction?

    @Transaction
    fun updateTransactionHash(
        oldId: TransactionId,
        walletId: WalletId,
        hash: String,
        updatedAt: Long = System.currentTimeMillis(),
    ) {
        val newId = TransactionId(oldId.chain, hash)
        if (oldId == newId) return
        val source = getTransaction(oldId, walletId) ?: return
        val target = getTransaction(newId, walletId)
        if (target != null) delete(newId, walletId)
        val transaction = (target ?: source).copy(recordId = source.recordId, id = newId, hash = hash, updatedAt = updatedAt)
        updateTransaction(transaction)
        if (transaction.type == TransactionType.Swap) copySwapMetadata(oldId.identifier, newId.identifier)
    }

    @Query(
        "UPDATE transactions SET state = :state, fee = COALESCE(:fee, fee), blockNumber = COALESCE(:blockNumber, blockNumber), " +
            "metadata = COALESCE(:metadata, metadata), estimatedConfirmationInSeconds = COALESCE(:confirmationEtaSeconds, estimatedConfirmationInSeconds), " +
            "updatedAt = :updatedAt WHERE id = :id AND walletId = :walletId"
    )
    fun updateTransactionState(
        id: TransactionId,
        walletId: WalletId,
        state: TransactionState,
        fee: String?,
        blockNumber: String?,
        metadata: String?,
        confirmationEtaSeconds: Long?,
        updatedAt: Long = System.currentTimeMillis(),
    ): Int

    @Insert(entity = DbTransactionSwapMetadata::class, onConflict = OnConflictStrategy.REPLACE)
    fun addSwapMetadata(metadata: List<DbTransactionSwapMetadata>)

    @Query("INSERT OR IGNORE INTO tx_swap_metadata (tx_id, from_asset_id, to_asset_id, from_amount, to_amount) SELECT :newId, from_asset_id, to_asset_id, from_amount, to_amount FROM tx_swap_metadata WHERE tx_id = :oldId")
    fun copySwapMetadata(oldId: String, newId: String)

    @Query("DELETE FROM tx_swap_metadata WHERE tx_id = :transactionId AND NOT EXISTS (SELECT 1 FROM transactions WHERE transactions.id = :transactionId)")
    fun deleteUnreferencedSwapMetadata(transactionId: String)

    @Query("""
        SELECT swap.from_asset_id AS fromAssetId, swap.to_asset_id AS toAssetId
        FROM tx_swap_metadata AS swap
        JOIN transactions AS tx ON tx.id = swap.tx_id
        WHERE tx.walletId = :walletId
        """)
    suspend fun getSwapPairs(walletId: String): List<DbSwapPair>

    @Query("DELETE FROM transactions WHERE state = :state")
    fun deleteByState(state: TransactionState)
}
