package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.RawQuery
import androidx.sqlite.db.SupportSQLiteQuery
import com.gemwallet.android.application.transactions.coordinators.TransactionsRequestFilter
import com.gemwallet.android.data.service.store.database.entities.DbAddress
import com.gemwallet.android.data.service.store.database.entities.DbAsset
import com.gemwallet.android.data.service.store.database.entities.DbPrice
import com.gemwallet.android.data.service.store.database.entities.DbTransaction
import com.gemwallet.android.data.service.store.database.entities.DbTransactionExtended
import com.gemwallet.android.data.service.store.database.entities.DbTxSwapMetadata
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow

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
    LEFT JOIN addresses as from_addr ON from_addr.chain = asset.chain AND from_addr.address = tx.owner
    LEFT JOIN addresses as to_addr ON to_addr.chain = asset.chain AND to_addr.address = tx.recipient
    WHERE tx.walletId = :walletId
"""

@Dao
interface TransactionsDao {

    @Insert(entity = DbTransaction::class, onConflict = OnConflictStrategy.REPLACE)
    fun insert(transactions: List<DbTransaction>)

    @Query("DELETE FROM transactions WHERE id = :id AND walletId = :walletId")
    fun delete(id: TransactionId, walletId: WalletId)

    @RawQuery(
        observedEntities = [
            DbTransaction::class,
            DbAsset::class,
            DbPrice::class,
            DbTxSwapMetadata::class,
            DbAddress::class,
        ]
    )
    fun getExtendedTransactions(query: SupportSQLiteQuery): Flow<List<DbTransactionExtended>>

    fun getExtendedTransactions(
        walletId: WalletId,
        filters: List<TransactionsRequestFilter> = emptyList(),
    ): Flow<List<DbTransactionExtended>> = getExtendedTransactions(buildExtendedTransactionsSql(walletId, filters).toSupportSQLiteQuery())

    @Query("SELECT COUNT(*) $EXTENDED_SOURCE AND tx.state IN (:states)")
    fun getTransactionsCount(walletId: WalletId, states: List<TransactionState>): Flow<Int?>

    @Query("SELECT $EXTENDED_COLUMNS $EXTENDED_SOURCE AND tx.id = :id")
    fun getExtendedTransaction(walletId: WalletId, id: TransactionId): Flow<DbTransactionExtended?>

    @Query("SELECT state FROM transactions WHERE id = :id AND walletId = :walletId")
    fun getTransactionState(id: TransactionId, walletId: WalletId): TransactionState?

    @Query("UPDATE transactions SET id = :newId, hash = :hash, updatedAt = :updatedAt WHERE id = :oldId AND walletId = :walletId")
    fun updateTransactionId(
        oldId: TransactionId,
        newId: TransactionId,
        walletId: WalletId,
        hash: String,
        updatedAt: Long = System.currentTimeMillis(),
    )

    @Query("UPDATE transactions SET state = :state, updatedAt = :updatedAt WHERE id = :id AND walletId = :walletId")
    fun updateState(id: TransactionId, walletId: WalletId, state: TransactionState, updatedAt: Long = System.currentTimeMillis())

    @Query("UPDATE transactions SET fee = :fee, updatedAt = :updatedAt WHERE id = :id AND walletId = :walletId")
    fun updateFee(id: TransactionId, walletId: WalletId, fee: String, updatedAt: Long = System.currentTimeMillis())

    @Query("UPDATE transactions SET metadata = :metadata, updatedAt = :updatedAt WHERE id = :id AND walletId = :walletId")
    fun updateMetadata(id: TransactionId, walletId: WalletId, metadata: String, updatedAt: Long = System.currentTimeMillis())

    @Insert(entity = DbTxSwapMetadata::class, onConflict = OnConflictStrategy.REPLACE)
    fun addSwapMetadata(metadata: List<DbTxSwapMetadata>)

    @Query("DELETE FROM tx_swap_metadata WHERE tx_id = :transactionId")
    fun deleteSwapMetadata(transactionId: String)

    @Query("UPDATE tx_swap_metadata SET tx_id = :newTransactionId WHERE tx_id = :oldTransactionId")
    fun updateSwapMetadataTransactionId(oldTransactionId: String, newTransactionId: String)

    @Query("DELETE FROM transactions WHERE state = :state")
    fun deleteByState(state: TransactionState)
}
