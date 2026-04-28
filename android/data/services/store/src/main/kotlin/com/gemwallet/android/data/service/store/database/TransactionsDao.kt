package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import com.gemwallet.android.data.service.store.database.entities.DbTransaction
import com.gemwallet.android.data.service.store.database.entities.DbTransactionExtended
import com.gemwallet.android.data.service.store.database.entities.DbTxSwapMetadata
import com.wallet.core.primitives.TransactionState
import kotlinx.coroutines.flow.Flow

const val SESSION_REQUEST = """SELECT accounts.address FROM accounts, session
    WHERE accounts.wallet_id = session.wallet_id AND session.id = 1"""
const val SESSION_CHAINS_REQUEST = """SELECT accounts.chain FROM accounts, session
    WHERE accounts.wallet_id = session.wallet_id AND session.id = 1"""
const val CURRENT_WALLET_REQUEST = """SELECT wallet_id FROM session WHERE session.id = 1"""

private const val EXTENDED_COLUMNS = """
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
    to_asset.type AS to_asset_type
"""

private const val EXTENDED_SOURCE = """
    FROM transactions as tx
    INNER JOIN asset ON tx.assetId = asset.id
    INNER JOIN asset as feeAsset ON tx.feeAssetId = feeAsset.id
    LEFT JOIN prices ON tx.assetId = prices.asset_id
    LEFT JOIN prices as feePrices ON tx.feeAssetId = feePrices.asset_id
    LEFT JOIN tx_swap_metadata as swap ON tx.id = swap.tx_id
    LEFT JOIN asset as from_asset ON swap.from_asset_id = from_asset.id
    LEFT JOIN asset as to_asset ON swap.to_asset_id = to_asset.id
    WHERE (tx.owner IN ($SESSION_REQUEST) OR tx.recipient IN ($SESSION_REQUEST))
        AND tx.walletId IN ($CURRENT_WALLET_REQUEST)
        AND tx.feeAssetId IN ($SESSION_CHAINS_REQUEST)
"""

@Dao
interface TransactionsDao {

    @Insert(entity = DbTransaction::class, onConflict = OnConflictStrategy.REPLACE)
    fun insert(transactions: List<DbTransaction>)

    @Query("DELETE FROM transactions WHERE id = :id AND walletId = :walletId")
    fun delete(id: String, walletId: String)

    @Query("SELECT $EXTENDED_COLUMNS $EXTENDED_SOURCE ORDER BY tx.createdAt DESC")
    fun getExtendedTransactions(): Flow<List<DbTransactionExtended>>

    @Query("SELECT $EXTENDED_COLUMNS $EXTENDED_SOURCE AND tx.state = :state ORDER BY tx.createdAt DESC")
    fun getExtendedTransactions(state: TransactionState): Flow<List<DbTransactionExtended>>

    @Query("SELECT COUNT(*) $EXTENDED_SOURCE AND tx.state = :state")
    fun getTransactionsCount(state: TransactionState): Flow<Int?>

    @Query("SELECT $EXTENDED_COLUMNS $EXTENDED_SOURCE AND tx.id = :id")
    fun getExtendedTransaction(id: String): Flow<DbTransactionExtended?>

    @Insert(entity = DbTxSwapMetadata::class, onConflict = OnConflictStrategy.REPLACE)
    fun addSwapMetadata(metadata: List<DbTxSwapMetadata>)

    @Query("SELECT * FROM tx_swap_metadata WHERE tx_id=:txId")
    fun getMetadata(txId: String): DbTxSwapMetadata?

    @Query("DELETE FROM transactions WHERE state = :state")
    fun deleteByState(state: TransactionState)
}
